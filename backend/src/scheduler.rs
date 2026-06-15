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
    maintenance_hours_per_day: f64,
    warmup_hours_per_batch: f64,
    slag_removal_minutes_per_ton: f64,
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
                maintenance_hours_per_day: 2.0,
                warmup_hours_per_batch: 1.5,
                slag_removal_minutes_per_ton: 8.0,
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
                maintenance_hours_per_day: 1.5,
                warmup_hours_per_batch: 1.0,
                slag_removal_minutes_per_ton: 4.0,
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
        self.register_furnace_with_maintenance(
            furnace_id,
            furnace_name,
            furnace_type,
            efficiency,
            max_output_per_hour,
            2.0,
            1.0,
            5.0,
        )
    }

    pub fn register_furnace_with_maintenance(
        &mut self,
        furnace_id: String,
        furnace_name: String,
        furnace_type: FurnaceType,
        efficiency: f64,
        max_output_per_hour: f64,
        maintenance_hours_per_day: f64,
        warmup_hours_per_batch: f64,
        slag_removal_minutes_per_ton: f64,
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
                maintenance_hours_per_day,
                warmup_hours_per_batch,
                slag_removal_minutes_per_ton,
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
            .map(|f| {
                let effective_hours = self.get_effective_production_hours(f, request.planning_hours);
                f.max_output_per_hour * effective_hours
            })
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

        let mut effective_hours_infeasible = false;

        for (furnace, output, fuel_type) in &allocation {
            let total_days = request.planning_hours / 24.0;
            let maintenance_hours = total_days * furnace.maintenance_hours_per_day;
            let effective_production_hours = self.get_effective_production_hours(furnace, request.planning_hours);

            if effective_production_hours <= 0.0 {
                effective_hours_infeasible = true;
                bottlenecks.push(format!("{} 有效生产时间不足（维护与预热耗时过长）", furnace.furnace_name));
                continue;
            }

            let hours = if *output > 0.0 {
                (output / furnace.max_output_per_hour).min(effective_production_hours)
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
            let maintenance_cost = maintenance_hours * 50.0;
            let production_cost = fuel_cost + ore_cost + labor_cost + maintenance_cost;

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
                    maintenance_hours,
                    effective_production_hours,
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

        let feasibility = !furnace_plans.is_empty()
            && total_output >= target_output * 0.5
            && !effective_hours_infeasible;

        if !feasibility {
            if effective_hours_infeasible {
                adjustments.push("维护时间过长，有效生产时间不足".to_string());
            } else {
                adjustments.push("资源严重不足，计划可行性较低".to_string());
            }
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

    fn get_effective_production_hours(&self, furnace: &FurnaceInfo, hours: f64) -> f64 {
        let total_days = hours / 24.0;
        let maintenance_downtime = total_days * furnace.maintenance_hours_per_day;
        let available_hours = hours - maintenance_downtime;

        if available_hours <= furnace.warmup_hours_per_batch {
            return 0.0;
        }

        let effective_hours = available_hours - furnace.warmup_hours_per_batch;

        let slag_hours_per_ton = furnace.slag_removal_minutes_per_ton / 60.0;
        let hours_per_ton_production = 1.0 / furnace.max_output_per_hour;
        let total_hours_per_ton = hours_per_ton_production + slag_hours_per_ton;

        if total_hours_per_ton <= 0.0 {
            return effective_hours;
        }

        let max_tonnage = effective_hours / total_hours_per_ton;
        (max_tonnage / furnace.max_output_per_hour).max(0.0)
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

                    let effective_hours = self.get_effective_production_hours(furnace, planning_hours);
                    let max_output = furnace.max_output_per_hour * effective_hours;
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
                    let effective_hours = self.get_effective_production_hours(furnace, planning_hours);
                    total_max += furnace.max_output_per_hour * effective_hours;
                }

                let target = target_output.min(total_max);
                let per_furnace = target / furnaces.len() as f64;

                for furnace in furnaces {
                    let effective_hours = self.get_effective_production_hours(furnace, planning_hours);
                    let output = per_furnace.min(furnace.max_output_per_hour * effective_hours);
                    if output > 0.0 {
                        allocations.push((furnace, output, cheapest_fuel));
                    }                }
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

                    let effective_hours = self.get_effective_production_hours(furnace, planning_hours);
                    let max_output = furnace.max_output_per_hour * effective_hours;
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

        let effective_production_hours = self.get_effective_production_hours(furnace, hours);
        let max_output = furnace.max_output_per_hour * effective_production_hours;
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

    fn rich_inventory() -> ResourceInventory {
        ResourceInventory {
            iron_ore_kg: 50000.0,
            charcoal_kg: 20000.0,
            coal_kg: 30000.0,
            coke_kg: 15000.0,
            wood_kg: 10000.0,
            limestone_kg: 5000.0,
            labor_hours: 2000.0,
        }
    }

    fn poor_inventory() -> ResourceInventory {
        ResourceInventory {
            iron_ore_kg: 10.0,
            charcoal_kg: 5.0,
            coal_kg: 5.0,
            coke_kg: 5.0,
            wood_kg: 5.0,
            limestone_kg: 5.0,
            labor_hours: 1.0,
        }
    }

    fn zero_inventory() -> ResourceInventory {
        ResourceInventory {
            iron_ore_kg: 0.0,
            charcoal_kg: 0.0,
            coal_kg: 0.0,
            coke_kg: 0.0,
            wood_kg: 0.0,
            limestone_kg: 0.0,
            labor_hours: 0.0,
        }
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = ProductionScheduler::new();
        let furnaces = scheduler.get_available_furnaces();
        assert_eq!(furnaces.len(), 2);
        assert!(furnaces.iter().any(|(id, _, _)| id == "HAN-001"));
        assert!(furnaces.iter().any(|(id, _, _)| id == "MING-001"));
    }

    #[test]
    fn test_get_furnace_info_existing() {
        let scheduler = ProductionScheduler::new();
        let info = scheduler.get_furnace_info("HAN-001");
        assert!(info.is_some());
        let (name, ftype, eff, max_out) = info.unwrap();
        assert!(!name.is_empty());
        assert_eq!(ftype, FurnaceType::HanChaogang);
        assert!(eff > 0.0 && eff <= 1.0);
        assert!(max_out > 0.0);
    }

    #[test]
    fn test_get_furnace_info_nonexistent() {
        let scheduler = ProductionScheduler::new();
        let info = scheduler.get_furnace_info("NONEXISTENT");
        assert!(info.is_none());
    }

    #[test]
    fn test_production_plan_normal() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(!plan.furnace_plans.is_empty());
        assert!(plan.total_iron_output_kg > 0.0);
        assert!(plan.feasibility);
        assert!(plan.total_cost > 0.0);
        assert!(plan.total_energy_efficiency > 0.0);
    }

    #[test]
    fn test_production_plan_quality_optimization() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "quality".to_string(),
            optimize_for: "quality".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.avg_iron_quality > 0.0);
        assert_eq!(plan.optimization_objective, "quality");
    }

    #[test]
    fn test_production_plan_cost_optimization() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "cost".to_string(),
            optimize_for: "cost".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.total_iron_output_kg > 0.0);
        assert_eq!(plan.optimization_objective, "cost");
    }

    #[test]
    fn test_production_plan_efficiency_optimization() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "efficiency".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.total_iron_output_kg > 0.0);
    }

    #[test]
    fn test_quality_optimization_prefers_efficient_furnace() {
        let scheduler = ProductionScheduler::new();
        let req = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "quality".to_string(),
            optimize_for: "quality".to_string(),
        };
        let plan = scheduler.create_plan(&req);
        let has_ming = plan.furnace_plans.iter().any(|p| p.furnace_id == "MING-001");
        assert!(has_ming, "quality optimization should use MING-001 (higher efficiency)");
    }

    #[test]
    fn test_plan_no_available_furnaces() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["NONEXISTENT".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.furnace_plans.is_empty());
        assert!(!plan.feasibility);
        assert!(plan.bottlenecks.iter().any(|b| b.contains("无可用")));
    }

    #[test]
    fn test_plan_empty_furnace_list() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec![],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.furnace_plans.is_empty());
        assert!(!plan.feasibility);
    }

    #[test]
    fn test_plan_zero_resources() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: zero_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.total_iron_output_kg == 0.0 || !plan.feasibility);
    }

    #[test]
    fn test_plan_poor_resources_bottleneck() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: poor_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(!plan.bottlenecks.is_empty());
    }

    #[test]
    fn test_plan_resource_usage_not_exceed_inventory() {
        let scheduler = ProductionScheduler::new();
        let inv = rich_inventory();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: inv.clone(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.resource_usage.iron_ore_kg <= inv.iron_ore_kg);
        assert!(plan.resource_usage.labor_hours <= inv.labor_hours);
    }

    #[test]
    fn test_plan_resource_remaining_non_negative() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.resource_remaining.iron_ore_kg >= 0.0);
        assert!(plan.resource_remaining.charcoal_kg >= 0.0);
        assert!(plan.resource_remaining.coal_kg >= 0.0);
        assert!(plan.resource_remaining.coke_kg >= 0.0);
        assert!(plan.resource_remaining.labor_hours >= 0.0);
    }

    #[test]
    fn test_plan_usage_plus_remaining_equals_inventory() {
        let scheduler = ProductionScheduler::new();
        let inv = rich_inventory();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: inv.clone(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!((plan.resource_usage.iron_ore_kg + plan.resource_remaining.iron_ore_kg - inv.iron_ore_kg).abs() < 1.0);
        assert!((plan.resource_usage.labor_hours + plan.resource_remaining.labor_hours - inv.labor_hours).abs() < 1.0);
    }

    #[test]
    fn test_plan_target_exceeds_capacity() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 1.0,
            target_iron_output_kg: 99999.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        let max_possible = 15.0 * 1.0 + 40.0 * 1.0;
        assert!(plan.total_iron_output_kg <= max_possible + 1.0);
        assert!(!plan.adjustments.is_empty());
    }

    #[test]
    fn test_plan_single_furnace() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.furnace_plans.len() <= 1);
    }

    #[test]
    fn test_plan_ming_blast_higher_output() {
        let scheduler = ProductionScheduler::new();
        let req_han = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 300.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let req_ming = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 300.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan_han = scheduler.create_plan(&req_han);
        let plan_ming = scheduler.create_plan(&req_ming);
        assert!(plan_ming.total_iron_output_kg >= plan_han.total_iron_output_kg);
    }

    #[test]
    fn test_plan_zero_hours() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 0.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.total_iron_output_kg == 0.0 || plan.furnace_plans.is_empty());
    }

    #[test]
    fn test_plan_zero_target() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 0.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.total_iron_output_kg >= 0.0);
    }

    #[test]
    fn test_estimate_production_normal() {
        let scheduler = ProductionScheduler::new();
        let (output, fuel, quality) =
            scheduler.estimate_production("HAN-001", FuelType::Charcoal, 8.0, 10000.0);
        assert!(output > 0.0);
        assert!(fuel > 0.0);
        assert!(quality > 0.0 && quality <= 1.0);
    }

    #[test]
    fn test_estimate_production_nonexistent_furnace() {
        let scheduler = ProductionScheduler::new();
        let (output, fuel, quality) =
            scheduler.estimate_production("NONEXISTENT", FuelType::Charcoal, 8.0, 10000.0);
        assert_eq!(output, 0.0);
        assert_eq!(fuel, 0.0);
        assert_eq!(quality, 0.0);
    }

    #[test]
    fn test_estimate_production_ore_limited() {
        let scheduler = ProductionScheduler::new();
        let (output_unlimited, _, _) =
            scheduler.estimate_production("MING-001", FuelType::Coke, 8.0, 100000.0);
        let (output_limited, _, _) =
            scheduler.estimate_production("MING-001", FuelType::Coke, 8.0, 10.0);
        assert!(output_limited < output_unlimited);
    }

    #[test]
    fn test_estimate_production_charcoal_highest_quality() {
        let scheduler = ProductionScheduler::new();
        let (_, _, q_charcoal) = scheduler.estimate_production("HAN-001", FuelType::Charcoal, 8.0, 10000.0);
        let (_, _, q_coal) = scheduler.estimate_production("HAN-001", FuelType::Coal, 8.0, 10000.0);
        let (_, _, q_wood) = scheduler.estimate_production("HAN-001", FuelType::Wood, 8.0, 10000.0);
        assert!(q_charcoal > q_coal);
        assert!(q_coal > q_wood);
    }

    #[test]
    fn test_estimate_production_zero_hours() {
        let scheduler = ProductionScheduler::new();
        let (output, _, _) = scheduler.estimate_production("HAN-001", FuelType::Charcoal, 0.0, 10000.0);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_register_new_furnace() {
        let mut scheduler = ProductionScheduler::new();
        scheduler.register_furnace(
            "TEST-001".to_string(),
            "测试炉".to_string(),
            FurnaceType::HanChaogang,
            0.60,
            10.0,
        );
        let furnaces = scheduler.get_available_furnaces();
        assert_eq!(furnaces.len(), 3);
        assert!(furnaces.iter().any(|(id, _, _)| id == "TEST-001"));

        let info = scheduler.get_furnace_info("TEST-001").unwrap();
        assert_eq!(info.0, "测试炉");
        assert_eq!(info.1, FurnaceType::HanChaogang);
    }

    #[test]
    fn test_plan_with_registered_furnace() {
        let mut scheduler = ProductionScheduler::new();
        scheduler.register_furnace(
            "TEST-001".to_string(),
            "测试炉".to_string(),
            FurnaceType::HanChaogang,
            0.60,
            10.0,
        );
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 50.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["TEST-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(plan.furnace_plans.iter().any(|p| p.furnace_id == "TEST-001"));
    }

    #[test]
    fn test_plan_furnace_plans_have_valid_fields() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "quality".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        for fp in &plan.furnace_plans {
            assert!(!fp.furnace_id.is_empty());
            assert!(!fp.furnace_name.is_empty());
            assert!(fp.operating_hours > 0.0);
            assert!(fp.iron_output_kg > 0.0);
            assert!(fp.fuel_required_kg > 0.0);
            assert!(fp.ore_required_kg > 0.0);
            assert!(fp.iron_quality_target > 0.0 && fp.iron_quality_target <= 1.0);
            assert!(fp.production_cost > 0.0);
        }
    }

    #[test]
    fn test_cost_optimization_uses_cheapest_fuel() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "cost".to_string(),
            optimize_for: "cost".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        for fp in &plan.furnace_plans {
            assert!(fp.fuel_type == FuelType::Coal,
                "cost optimization should prefer coal (cheapest), got {:?}", fp.fuel_type);
        }
    }

    #[test]
    fn test_plan_default_inventory_no_fuel() {
        let scheduler = ProductionScheduler::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: zero_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = scheduler.create_plan(&request);
        assert!(!plan.feasibility || plan.total_iron_output_kg == 0.0);
    }
}
