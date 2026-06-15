use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::{
    FurnaceProductionPlan, FurnaceType, FuelType, ProductionPlan, ResourceInventory,
    SchedulingRequest,
};

struct FurnaceInfo {
    furnace_id: String,
    furnace_name: String,
    furnace_type: FurnaceType,
    efficiency: f64,
    max_output_per_hour: f64,
    labor_per_hour: f64,
    prefer_fuel: FuelType,
}

pub struct ProductionScheduler {
    furnaces: HashMap<String, FurnaceInfo>,
    default_labor_per_hour: f64,
    default_ore_ratio: f64,
    default_fuel_ratio: f64,
}

impl ProductionScheduler {
    pub fn new() -> Self {
        let mut furnaces = HashMap::new();

        furnaces.insert(
            "HAN-001".to_string(),
            FurnaceInfo {
                furnace_id: "HAN-001".to_string(),
                furnace_name: "汉代炒钢炉一号".to_string(),
                furnace_type: FurnaceType::HanChaogang,
                efficiency: 0.70,
                max_output_per_hour: 15.0,
                labor_per_hour: 3.0,
                prefer_fuel: FuelType::Charcoal,
            },
        );

        furnaces.insert(
            "MING-001".to_string(),
            FurnaceInfo {
                furnace_id: "MING-001".to_string(),
                furnace_name: "明代高炉一号".to_string(),
                furnace_type: FurnaceType::MingBlast,
                efficiency: 0.85,
                max_output_per_hour: 40.0,
                labor_per_hour: 5.0,
                prefer_fuel: FuelType::Coke,
            },
        );

        Self {
            furnaces,
            default_labor_per_hour: 2.5,
            default_ore_ratio: 1.8,
            default_fuel_ratio: 1.2,
        }
    }

    pub fn register_furnace(
        &mut self,
        furnace_id: String,
        furnace_name: String,
        furnace_type: FurnaceType,
        efficiency: f64,
        max_output_per_hour: f64,
    ) {
        let prefer_fuel = match furnace_type {
            FurnaceType::HanChaogang => FuelType::Charcoal,
            FurnaceType::MingBlast => FuelType::Coal,
        };

        self.furnaces.insert(
            furnace_id.clone(),
            FurnaceInfo {
                furnace_id,
                furnace_name,
                furnace_type,
                efficiency,
                max_output_per_hour,
                labor_per_hour: self.default_labor_per_hour,
                prefer_fuel,
            },
        );
    }

    pub fn create_plan(&self, request: &SchedulingRequest) -> ProductionPlan {
        info!(
            "Creating production plan: {} hours, target {} kg iron",
            request.planning_hours, request.target_iron_output_kg
        );

        let available_furnaces: Vec<&FurnaceInfo> = request
            .available_furnaces
            .iter()
            .filter_map(|id| self.furnaces.get(id))
            .collect();

        if available_furnaces.is_empty() {
            warn!("No available furnaces for scheduling");
            return self.create_empty_plan(request);
        }

        let optimize_for = request.optimize_for.as_str();

        let mut furnace_plans = Vec::new();
        let mut resource_usage = ResourceInventory::default();
        let mut adjustments = Vec::new();
        let mut bottlenecks = Vec::new();

        let total_max_output = available_furnaces
            .iter()
            .map(|f| f.max_output_per_hour * request.planning_hours)
            .sum::<f64>();

        let target_output = request.target_iron_output_kg.min(total_max_output);

        if target_output < request.target_iron_output_kg {
            adjustments.push(format!(
                "目标产量过高，已调整为最大可能产量 {:.0} kg",
                total_max_output
            ));
            bottlenecks.push("冶炼炉产能不足".to_string());
        }

        let allocation = self.allocate_production(
            &available_furnaces,
            target_output,
            &request.inventory,
            optimize_for,
            request.planning_hours,
        );

        let mut remaining_ore = request.inventory.iron_ore_kg;
        let mut remaining_charcoal = request.inventory.charcoal_kg;
        let mut remaining_coal = request.inventory.coal_kg;
        let mut remaining_coke = request.inventory.coke_kg;
        let mut remaining_labor = request.inventory.labor_hours;
        let mut total_output = 0.0;
        let mut total_cost = 0.0;
        let mut total_weighted_quality = 0.0;

        for (furnace, output, fuel_type) in &allocation {
            let hours = if *output > 0.0 {
                (output / furnace.max_output_per_hour).min(request.planning_hours)
            } else {
                0.0
            };

            if hours <= 0.0 {
                continue;
            }

            let ore_required = output * self.default_ore_ratio;
            let fuel_required = output * self.default_fuel_ratio;
            let labor_required = hours * furnace.labor_per_hour;

            let fuel_cost_per_kg = match fuel_type {
                FuelType::Charcoal => 2.5,
                FuelType::Coal => 0.8,
                FuelType::Coke => 1.8,
                FuelType::Wood => 0.3,
            };

            let fuel_cost = fuel_required * fuel_cost_per_kg;
            let ore_cost = ore_required * 0.5;
            let labor_cost = labor_required * 20.0;
            let production_cost = fuel_cost + ore_cost + labor_cost;

            let quality_score = match fuel_type {
                FuelType::Charcoal => 0.85,
                FuelType::Coal => 0.70,
                FuelType::Coke => 0.78,
                FuelType::Wood => 0.60,
            } * furnace.efficiency;

            if remaining_ore >= ore_required
                && remaining_labor >= labor_required
                && self.check_fuel(fuel_type, fuel_required, remaining_charcoal, remaining_coal, remaining_coke)
            {
                remaining_ore -= ore_required;
                remaining_labor -= labor_required;

                match fuel_type {
                    FuelType::Charcoal => remaining_charcoal -= fuel_required,
                    FuelType::Coal => remaining_coal -= fuel_required,
                    FuelType::Coke => remaining_coke -= fuel_required,
                    _ => {}
                }

                total_output += output;
                total_cost += production_cost;
                total_weighted_quality += quality_score * output;

                furnace_plans.push(FurnaceProductionPlan {
                    furnace_id: furnace.furnace_id.clone(),
                    furnace_name: furnace.furnace_name.clone(),
                    furnace_type: furnace.furnace_type,
                    fuel_type: *fuel_type,
                    operating_hours: hours,
                    target_temp: match furnace.furnace_type {
                        FurnaceType::HanChaogang => 1250.0,
                        FurnaceType::MingBlast => 1400.0,
                    },
                    iron_output_kg: *output,
                    iron_quality_target: quality_score,
                    fuel_required_kg: fuel_required,
                    ore_required_kg: ore_required,
                    labor_required_hours: labor_required,
                    production_cost,
                    start_hour: 0.0,
                    end_hour: hours,
                    status: "scheduled".to_string(),
                });
            } else {
                if remaining_ore < ore_required {
                    bottlenecks.push(format!("{} 铁矿石不足", furnace.furnace_name));
                }
                if remaining_labor < labor_required {
                    bottlenecks.push(format!("{} 劳动力不足", furnace.furnace_name));
                }
                adjustments.push(format!(
                    "{} 因资源限制，产量从 {:.0} kg 缩减",
                    furnace.furnace_name, output
                ));
            }
        }

        resource_usage.iron_ore_kg = request.inventory.iron_ore_kg - remaining_ore;
        resource_usage.charcoal_kg = request.inventory.charcoal_kg - remaining_charcoal;
        resource_usage.coal_kg = request.inventory.coal_kg - remaining_coal;
        resource_usage.coke_kg = request.inventory.coke_kg - remaining_coke;
        resource_usage.wood_kg = 0.0;
        resource_usage.limestone_kg = request.inventory.limestone_kg * 0.1;
        resource_usage.labor_hours = request.inventory.labor_hours - remaining_labor;

        let resource_remaining = ResourceInventory {
            iron_ore_kg: remaining_ore,
            charcoal_kg: remaining_charcoal,
            coal_kg: remaining_coal,
            coke_kg: remaining_coke,
            wood_kg: request.inventory.wood_kg,
            limestone_kg: request.inventory.limestone_kg - resource_usage.limestone_kg,
            labor_hours: remaining_labor,
        };

        let feasibility = !furnace_plans.is_empty() && total_output >= target_output * 0.5;

        if !feasibility {
            adjustments.push("资源严重不足，计划可行性较低".to_string());
        }

        let avg_quality = if total_output > 0.0 {
            total_weighted_quality / total_output
        } else {
            0.0
        };

        let total_efficiency = if !furnace_plans.is_empty() {
            furnace_plans
                .iter()
                .map(|p| {
                    let info = self.furnaces.get(&p.furnace_id).unwrap();
                    info.efficiency
                })
                .sum::<f64>()
                / furnace_plans.len() as f64
        } else {
            0.0
        };

        ProductionPlan {
            plan_id: Uuid::new_v4(),
            created_at: Utc::now(),
            planning_hours: request.planning_hours,
            total_iron_output_kg: total_output,
            total_cost,
            total_energy_efficiency: total_efficiency,
            avg_iron_quality: avg_quality,
            furnace_plans,
            resource_usage,
            resource_remaining,
            optimization_objective: optimize_for.to_string(),
            bottlenecks,
            feasibility,
            adjustments,
        }
    }

    fn check_fuel(
        &self,
        fuel_type: &FuelType,
        required: f64,
        charcoal: f64,
        coal: f64,
        coke: f64,
    ) -> bool {
        match fuel_type {
            FuelType::Charcoal => charcoal >= required,
            FuelType::Coal => coal >= required,
            FuelType::Coke => coke >= required,
            FuelType::Wood => true,
        }
    }

    fn allocate_production<'a>(
        &self,
        furnaces: &'a [&'a FurnaceInfo],
        target_output: f64,
        inventory: &ResourceInventory,
        optimize_for: &str,
        planning_hours: f64,
    ) -> Vec<(&'a FurnaceInfo, f64, FuelType)> {
        let mut allocations: Vec<(&'a FurnaceInfo, f64, FuelType)> = Vec::new();

        match optimize_for {
            "quality" => {
                let mut sorted_furnaces: Vec<&FurnaceInfo> = furnaces.iter().copied().collect();
                sorted_furnaces.sort_by(|a, b| b.efficiency.partial_cmp(&a.efficiency).unwrap());

                let mut remaining_output = target_output;

                for furnace in &sorted_furnaces {
                    if remaining_output <= 0.0 {
                        break;
                    }

                    let max_output = furnace.max_output_per_hour * planning_hours;
                    let output = max_output.min(remaining_output);

                    if output > 0.0 {
                        allocations.push((furnace, output, furnace.prefer_fuel));
                        remaining_output -= output;
                    }
                }
            }

            "cost" => {
                let fuel_options = [FuelType::Coal, FuelType::Coke, FuelType::Charcoal];

                let mut cheapest_fuel = FuelType::Charcoal;
                let mut cheapest_cost = f64::MAX;

                for fuel in &fuel_options {
                    let fuel_available = match fuel {
                        FuelType::Charcoal => inventory.charcoal_kg,
                        FuelType::Coal => inventory.coal_kg,
                        FuelType::Coke => inventory.coke_kg,
                        FuelType::Wood => inventory.wood_kg,
                    };

                    if fuel_available <= 0.0 {
                        continue;
                    }

                    let cost_per_kg = match fuel {
                        FuelType::Charcoal => 2.5,
                        FuelType::Coal => 0.8,
                        FuelType::Coke => 1.8,
                        FuelType::Wood => 0.3,
                    };

                    if cost_per_kg < cheapest_cost {
                        cheapest_cost = cost_per_kg;
                        cheapest_fuel = *fuel;
                    }
                }

                let mut total_max = 0.0;
                for furnace in furnaces {
                    total_max += furnace.max_output_per_hour * planning_hours;
                }

                let target = target_output.min(total_max);
                let per_furnace = target / furnaces.len() as f64;

                for furnace in furnaces {
                    let output = per_furnace.min(furnace.max_output_per_hour * planning_hours);
                    if output > 0.0 {
                        allocations.push((furnace, output, cheapest_fuel));
                    }
                }
            }

            "efficiency" | _ => {
                let mut sorted_furnaces: Vec<&FurnaceInfo> = furnaces.iter().copied().collect();
                sorted_furnaces
                    .sort_by(|a, b| b.max_output_per_hour.partial_cmp(&a.max_output_per_hour).unwrap());

                let mut remaining_output = target_output;

                for furnace in &sorted_furnaces {
                    if remaining_output <= 0.0 {
                        break;
                    }

                    let max_output = furnace.max_output_per_hour * planning_hours;
                    let output = max_output.min(remaining_output);

                    if output > 0.0 {
                        let fuel = if inventory.coal_kg > output * self.default_fuel_ratio {
                            FuelType::Coal
                        } else if inventory.charcoal_kg > output * self.default_fuel_ratio {
                            FuelType::Charcoal
                        } else {
                            FuelType::Wood
                        };

                        allocations.push((furnace, output, fuel));
                        remaining_output -= output;
                    }
                }
            }
        }

        allocations
    }

    fn create_empty_plan(&self, request: &SchedulingRequest) -> ProductionPlan {
        ProductionPlan {
            plan_id: Uuid::new_v4(),
            created_at: Utc::now(),
            planning_hours: request.planning_hours,
            total_iron_output_kg: 0.0,
            total_cost: 0.0,
            total_energy_efficiency: 0.0,
            avg_iron_quality: 0.0,
            furnace_plans: Vec::new(),
            resource_usage: ResourceInventory::default(),
            resource_remaining: request.inventory.clone(),
            optimization_objective: request.optimize_for.clone(),
            bottlenecks: vec!["无可用冶炼炉".to_string()],
            feasibility: false,
            adjustments: vec!["请添加可用冶炼炉".to_string()],
        }
    }

    pub fn get_available_furnaces(&self) -> Vec<(String, String, FurnaceType)> {
        self.furnaces
            .values()
            .map(|f| (f.furnace_id.clone(), f.furnace_name.clone(), f.furnace_type))
            .collect()
    }

    pub fn get_furnace_info(&self, furnace_id: &str) -> Option<(String, FurnaceType, f64, f64)> {
        self.furnaces.get(furnace_id).map(|f| {
            (
                f.furnace_name.clone(),
                f.furnace_type,
                f.efficiency,
                f.max_output_per_hour,
            )
        })
    }

    pub fn estimate_production(
        &self,
        furnace_id: &str,
        fuel_type: FuelType,
        hours: f64,
        ore_kg: f64,
    ) -> (f64, f64, f64) {
        let furnace = match self.furnaces.get(furnace_id) {
            Some(f) => f,
            None => return (0.0, 0.0, 0.0),
        };

        let max_output = furnace.max_output_per_hour * hours;
        let ore_limited = ore_kg / self.default_ore_ratio;
        let actual_output = max_output.min(ore_limited);

        let fuel_needed = actual_output * self.default_fuel_ratio;

        let quality = match fuel_type {
            FuelType::Charcoal => 0.85,
            FuelType::Coal => 0.70,
            FuelType::Coke => 0.78,
            FuelType::Wood => 0.60,
        } * furnace.efficiency;

        (actual_output, fuel_needed, quality)
    }
}

impl Default for ProductionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = ProductionScheduler::new();
        let furnaces = scheduler.get_available_furnaces();
        assert_eq!(furnaces.len(), 2);
    }

    #[test]
    fn test_production_plan() {
        let scheduler = ProductionScheduler::new();

        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: ResourceInventory::default(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };

        let plan = scheduler.create_plan(&request);
        assert!(!plan.furnace_plans.is_empty());
        assert!(plan.total_iron_output_kg > 0.0);
        assert!(plan.feasibility);
    }

    #[test]
    fn test_production_plan_quality_optimization() {
        let scheduler = ProductionScheduler::new();

        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: ResourceInventory::default(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "quality".to_string(),
            optimize_for: "quality".to_string(),
        };

        let plan = scheduler.create_plan(&request);
        assert!(plan.avg_iron_quality > 0.0);
    }

    #[test]
    fn test_estimate_production() {
        let scheduler = ProductionScheduler::new();
        let (output, fuel, quality) =
            scheduler.estimate_production("HAN-001", FuelType::Charcoal, 8.0, 100.0);

        assert!(output > 0.0);
        assert!(fuel > 0.0);
        assert!(quality > 0.0 && quality <= 1.0);
    }
}
