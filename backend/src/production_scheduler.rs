use crate::models::{
    FurnaceType, FuelType, ProductionPlan, ResourceInventory, SchedulingRequest,
};
use crate::scheduler::ProductionScheduler;

pub struct ProductionPlanningEngine {
    scheduler: ProductionScheduler,
}

impl ProductionPlanningEngine {
    pub fn new() -> Self {
        Self {
            scheduler: ProductionScheduler::new(),
        }
    }

    pub fn create_plan(&self, request: &SchedulingRequest) -> ProductionPlan {
        self.scheduler.create_plan(request)
    }

    pub fn estimate_production(
        &self,
        furnace_id: &str,
        fuel: FuelType,
        hours: f64,
        ore_kg: f64,
    ) -> (f64, f64, f64) {
        self.scheduler
            .estimate_production(furnace_id, fuel, hours, ore_kg)
    }

    pub fn get_available_furnaces(&self) -> Vec<String> {
        self.scheduler
            .get_available_furnaces()
            .into_iter()
            .map(|(id, _, _)| id)
            .collect()
    }

    pub fn get_furnace_info(
        &self,
        furnace_id: &str,
    ) -> Option<(String, FurnaceType, f64, f64)> {
        self.scheduler.get_furnace_info(furnace_id)
    }

    pub fn register_furnace(
        &mut self,
        id: String,
        name: String,
        ftype: FurnaceType,
        eff: f64,
        max_output: f64,
    ) {
        self.scheduler
            .register_furnace(id, name, ftype, eff, max_output)
    }

    pub fn calculate_effective_hours(
        total_hours: f64,
        maintenance_per_day: f64,
        warmup: f64,
        slag_minutes_per_ton: f64,
    ) -> f64 {
        let total_days = total_hours / 24.0;
        let maintenance_downtime = total_days * maintenance_per_day;
        let available_hours = total_hours - maintenance_downtime;

        if available_hours <= warmup {
            return 0.0;
        }

        let after_warmup = available_hours - warmup;
        let slag_hours_per_hour_production = slag_minutes_per_ton / 60.0;
        after_warmup / (1.0 + slag_hours_per_hour_production)
    }

    pub fn check_feasibility(
        &self,
        inventory: &ResourceInventory,
        furnaces: &[String],
        plan_hours: f64,
    ) -> (bool, Vec<String>) {
        let mut total_max_capacity = 0.0;
        for furnace_id in furnaces {
            if let Some((_, _, _, max_output_per_hour)) = self.get_furnace_info(furnace_id) {
                total_max_capacity += max_output_per_hour * plan_hours;
            }
        }

        let target = if total_max_capacity > 0.0 {
            total_max_capacity
        } else {
            1.0
        };

        let request = SchedulingRequest {
            planning_hours: plan_hours,
            target_iron_output_kg: target,
            inventory: inventory.clone(),
            available_furnaces: furnaces.to_vec(),
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };

        let plan = self.create_plan(&request);
        (plan.feasibility, plan.bottlenecks.clone())
    }
}

impl Default for ProductionPlanningEngine {
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
    fn test_engine_creation() {
        let engine = ProductionPlanningEngine::new();
        let furnaces = engine.get_available_furnaces();
        assert_eq!(furnaces.len(), 2);
        assert!(furnaces.iter().any(|id| id == "HAN-001"));
        assert!(furnaces.iter().any(|id| id == "MING-001"));
    }

    #[test]
    fn test_scheduler_creation() {
        let engine = ProductionPlanningEngine::new();
        let furnaces = engine.get_available_furnaces();
        assert_eq!(furnaces.len(), 2);
        assert!(furnaces.iter().any(|id| id == "HAN-001"));
        assert!(furnaces.iter().any(|id| id == "MING-001"));
    }

    #[test]
    fn test_get_furnace_info_existing() {
        let engine = ProductionPlanningEngine::new();
        let info = engine.get_furnace_info("HAN-001");
        assert!(info.is_some());
        let (name, ftype, eff, max_out) = info.unwrap();
        assert!(!name.is_empty());
        assert_eq!(ftype, FurnaceType::HanChaogang);
        assert!(eff > 0.0 && eff <= 1.0);
        assert!(max_out > 0.0);
    }

    #[test]
    fn test_get_furnace_info_nonexistent() {
        let engine = ProductionPlanningEngine::new();
        let info = engine.get_furnace_info("NONEXISTENT");
        assert!(info.is_none());
    }

    #[test]
    fn test_production_plan_normal() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(!plan.furnace_plans.is_empty());
        assert!(plan.total_iron_output_kg > 0.0);
        assert!(plan.feasibility);
        assert!(plan.total_cost > 0.0);
        assert!(plan.total_energy_efficiency > 0.0);
    }

    #[test]
    fn test_production_plan_quality_optimization() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "quality".to_string(),
            optimize_for: "quality".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.avg_iron_quality > 0.0);
        assert_eq!(plan.optimization_objective, "quality");
    }

    #[test]
    fn test_production_plan_cost_optimization() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "cost".to_string(),
            optimize_for: "cost".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.total_iron_output_kg > 0.0);
        assert_eq!(plan.optimization_objective, "cost");
    }

    #[test]
    fn test_production_plan_efficiency_optimization() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "efficiency".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.total_iron_output_kg > 0.0);
    }

    #[test]
    fn test_quality_optimization_prefers_efficient_furnace() {
        let engine = ProductionPlanningEngine::new();
        let req = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "quality".to_string(),
            optimize_for: "quality".to_string(),
        };
        let plan = engine.create_plan(&req);
        let has_ming = plan.furnace_plans.iter().any(|p| p.furnace_id == "MING-001");
        assert!(
            has_ming,
            "quality optimization should use MING-001 (higher efficiency)"
        );
    }

    #[test]
    fn test_plan_no_available_furnaces() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["NONEXISTENT".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.furnace_plans.is_empty());
        assert!(!plan.feasibility);
        assert!(plan.bottlenecks.iter().any(|b| b.contains("无可用")));
    }

    #[test]
    fn test_plan_empty_furnace_list() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec![],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.furnace_plans.is_empty());
        assert!(!plan.feasibility);
    }

    #[test]
    fn test_plan_zero_resources() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: zero_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.total_iron_output_kg == 0.0 || !plan.feasibility);
    }

    #[test]
    fn test_plan_poor_resources_bottleneck() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: poor_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(!plan.bottlenecks.is_empty());
    }

    #[test]
    fn test_plan_resource_usage_not_exceed_inventory() {
        let engine = ProductionPlanningEngine::new();
        let inv = rich_inventory();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: inv.clone(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.resource_usage.iron_ore_kg <= inv.iron_ore_kg);
        assert!(plan.resource_usage.labor_hours <= inv.labor_hours);
    }

    #[test]
    fn test_plan_resource_remaining_non_negative() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.resource_remaining.iron_ore_kg >= 0.0);
        assert!(plan.resource_remaining.charcoal_kg >= 0.0);
        assert!(plan.resource_remaining.coal_kg >= 0.0);
        assert!(plan.resource_remaining.coke_kg >= 0.0);
        assert!(plan.resource_remaining.labor_hours >= 0.0);
    }

    #[test]
    fn test_plan_usage_plus_remaining_equals_inventory() {
        let engine = ProductionPlanningEngine::new();
        let inv = rich_inventory();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: inv.clone(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(
            (plan.resource_usage.iron_ore_kg + plan.resource_remaining.iron_ore_kg
                - inv.iron_ore_kg)
                .abs()
                < 1.0
        );
        assert!(
            (plan.resource_usage.labor_hours + plan.resource_remaining.labor_hours
                - inv.labor_hours)
                .abs()
                < 1.0
        );
    }

    #[test]
    fn test_plan_target_exceeds_capacity() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 1.0,
            target_iron_output_kg: 99999.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        let max_possible = 15.0 * 1.0 + 40.0 * 1.0;
        assert!(plan.total_iron_output_kg <= max_possible + 1.0);
        assert!(!plan.adjustments.is_empty());
    }

    #[test]
    fn test_plan_single_furnace() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 100.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.furnace_plans.len() <= 1);
    }

    #[test]
    fn test_plan_ming_blast_higher_output() {
        let engine = ProductionPlanningEngine::new();
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
        let plan_han = engine.create_plan(&req_han);
        let plan_ming = engine.create_plan(&req_ming);
        assert!(plan_ming.total_iron_output_kg >= plan_han.total_iron_output_kg);
    }

    #[test]
    fn test_plan_zero_hours() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 0.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.total_iron_output_kg == 0.0 || plan.furnace_plans.is_empty());
    }

    #[test]
    fn test_plan_zero_target() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 0.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(plan.total_iron_output_kg >= 0.0);
    }

    #[test]
    fn test_estimate_production_normal() {
        let engine = ProductionPlanningEngine::new();
        let (output, fuel, quality) =
            engine.estimate_production("HAN-001", FuelType::Charcoal, 8.0, 10000.0);
        assert!(output > 0.0);
        assert!(fuel > 0.0);
        assert!(quality > 0.0 && quality <= 1.0);
    }

    #[test]
    fn test_estimate_production_nonexistent_furnace() {
        let engine = ProductionPlanningEngine::new();
        let (output, fuel, quality) =
            engine.estimate_production("NONEXISTENT", FuelType::Charcoal, 8.0, 10000.0);
        assert_eq!(output, 0.0);
        assert_eq!(fuel, 0.0);
        assert_eq!(quality, 0.0);
    }

    #[test]
    fn test_estimate_production_ore_limited() {
        let engine = ProductionPlanningEngine::new();
        let (output_unlimited, _, _) =
            engine.estimate_production("MING-001", FuelType::Coke, 8.0, 100000.0);
        let (output_limited, _, _) =
            engine.estimate_production("MING-001", FuelType::Coke, 8.0, 10.0);
        assert!(output_limited < output_unlimited);
    }

    #[test]
    fn test_estimate_production_charcoal_highest_quality() {
        let engine = ProductionPlanningEngine::new();
        let (_, _, q_charcoal) =
            engine.estimate_production("HAN-001", FuelType::Charcoal, 8.0, 10000.0);
        let (_, _, q_coal) =
            engine.estimate_production("HAN-001", FuelType::Coal, 8.0, 10000.0);
        let (_, _, q_wood) =
            engine.estimate_production("HAN-001", FuelType::Wood, 8.0, 10000.0);
        assert!(q_charcoal > q_coal);
        assert!(q_coal > q_wood);
    }

    #[test]
    fn test_estimate_production_zero_hours() {
        let engine = ProductionPlanningEngine::new();
        let (output, _, _) =
            engine.estimate_production("HAN-001", FuelType::Charcoal, 0.0, 10000.0);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_register_new_furnace() {
        let mut engine = ProductionPlanningEngine::new();
        engine.register_furnace(
            "TEST-001".to_string(),
            "测试炉".to_string(),
            FurnaceType::HanChaogang,
            0.60,
            10.0,
        );
        let furnaces = engine.get_available_furnaces();
        assert_eq!(furnaces.len(), 3);
        assert!(furnaces.iter().any(|id| id == "TEST-001"));

        let info = engine.get_furnace_info("TEST-001").unwrap();
        assert_eq!(info.0, "测试炉");
        assert_eq!(info.1, FurnaceType::HanChaogang);
    }

    #[test]
    fn test_plan_with_registered_furnace() {
        let mut engine = ProductionPlanningEngine::new();
        engine.register_furnace(
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
        let plan = engine.create_plan(&request);
        assert!(plan
            .furnace_plans
            .iter()
            .any(|p| p.furnace_id == "TEST-001"));
    }

    #[test]
    fn test_plan_furnace_plans_have_valid_fields() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "quality".to_string(),
        };
        let plan = engine.create_plan(&request);
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
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: rich_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "cost".to_string(),
            optimize_for: "cost".to_string(),
        };
        let plan = engine.create_plan(&request);
        for fp in &plan.furnace_plans {
            assert!(
                fp.fuel_type == FuelType::Coal,
                "cost optimization should prefer coal (cheapest), got {:?}",
                fp.fuel_type
            );
        }
    }

    #[test]
    fn test_plan_default_inventory_no_fuel() {
        let engine = ProductionPlanningEngine::new();
        let request = SchedulingRequest {
            planning_hours: 8.0,
            target_iron_output_kg: 200.0,
            inventory: zero_inventory(),
            available_furnaces: vec!["HAN-001".to_string(), "MING-001".to_string()],
            priority: "balanced".to_string(),
            optimize_for: "efficiency".to_string(),
        };
        let plan = engine.create_plan(&request);
        assert!(!plan.feasibility || plan.total_iron_output_kg == 0.0);
    }

    #[test]
    fn test_calculate_effective_hours_normal() {
        let effective =
            ProductionPlanningEngine::calculate_effective_hours(24.0, 2.0, 1.5, 5.0);
        assert!(effective > 0.0);
        assert!(effective < 24.0);
    }

    #[test]
    fn test_calculate_effective_hours_zero_total() {
        let effective =
            ProductionPlanningEngine::calculate_effective_hours(0.0, 2.0, 1.5, 5.0);
        assert_eq!(effective, 0.0);
    }

    #[test]
    fn test_calculate_effective_hours_warmup_exceeds_available() {
        let effective =
            ProductionPlanningEngine::calculate_effective_hours(1.5, 2.0, 1.5, 5.0);
        assert_eq!(effective, 0.0);
    }

    #[test]
    fn test_calculate_effective_hours_no_maintenance_no_slag() {
        let effective =
            ProductionPlanningEngine::calculate_effective_hours(24.0, 0.0, 0.0, 0.0);
        assert_eq!(effective, 24.0);
    }

    #[test]
    fn test_check_feasibility_rich_inventory() {
        let engine = ProductionPlanningEngine::new();
        let furnaces = vec!["HAN-001".to_string(), "MING-001".to_string()];
        let (feasible, _bottlenecks) =
            engine.check_feasibility(&rich_inventory(), &furnaces, 8.0);
        assert!(feasible);
    }

    #[test]
    fn test_check_feasibility_zero_inventory() {
        let engine = ProductionPlanningEngine::new();
        let furnaces = vec!["HAN-001".to_string(), "MING-001".to_string()];
        let (feasible, bottlenecks) =
            engine.check_feasibility(&zero_inventory(), &furnaces, 8.0);
        assert!(!feasible);
        assert!(!bottlenecks.is_empty());
    }

    #[test]
    fn test_check_feasibility_no_furnaces() {
        let engine = ProductionPlanningEngine::new();
        let furnaces = vec![];
        let (feasible, bottlenecks) =
            engine.check_feasibility(&rich_inventory(), &furnaces, 8.0);
        assert!(!feasible);
        assert!(!bottlenecks.is_empty());
    }
}
