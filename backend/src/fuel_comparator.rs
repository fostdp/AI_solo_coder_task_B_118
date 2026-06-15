use crate::fuel::FuelSystem;
use crate::models::{
    FuelComparisonRequest, FuelComparisonResult, FuelDataSource, FuelProperties, FuelType,
    IronQualityMetrics, SlagComposition,
};

pub struct FuelComparator {
    system: FuelSystem,
}

impl FuelComparator {
    pub fn new() -> Self {
        Self {
            system: FuelSystem::new(),
        }
    }

    pub fn get_all_fuel_properties(&self) -> Vec<FuelProperties> {
        self.system
            .all_fuel_properties()
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn get_fuel_properties(&self, fuel: FuelType) -> Option<FuelProperties> {
        self.system
            .get_fuel_properties(fuel)
            .cloned()
    }

    pub fn compare_fuels(&self, request: &FuelComparisonRequest) -> FuelComparisonResult {
        self.system.compare_fuels(request)
    }

    pub fn get_iron_quality(
        &self,
        fuel: FuelType,
        temp_c: f64,
        co_factor: f64,
        _ore_type: &str,
    ) -> IronQualityMetrics {
        self.system.calculate_iron_quality(fuel, temp_c, co_factor)
    }

    pub fn get_slag_composition(&self, fuel: FuelType, ore_type: &str) -> SlagComposition {
        self.system.calculate_slag_composition(fuel, ore_type, 1300.0)
    }

    pub fn calculate_combustion_rate(
        &self,
        fuel: FuelType,
        temp_c: f64,
        air_flow: f64,
    ) -> f64 {
        self.system.calculate_combustion_rate(fuel, temp_c, air_flow)
    }

    pub fn calculate_temp_effect(
        &self,
        fuel: FuelType,
        base_temp: f64,
        burn_rate: f64,
        loss_coeff: f64,
    ) -> f64 {
        self.system
            .calculate_temp_effect(fuel, base_temp, burn_rate, loss_coeff)
    }

    pub fn calculate_heat_release_rate(
        &self,
        fuel: FuelType,
        burn_rate: f64,
        efficiency: f64,
    ) -> f64 {
        self.system
            .calculate_heat_release_rate(fuel, burn_rate, efficiency)
    }

    pub fn get_literature_source(&self, fuel: FuelType) -> Option<FuelDataSource> {
        self.system
            .get_fuel_properties(fuel)
            .map(|props| props.data_source.clone())
    }
}

impl Default for FuelComparator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel::FuelSystem;
    use crate::models::FurnaceType;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_fuel_properties_charcoal() {
        let system = FuelSystem::new();
        let charcoal = system.get_fuel_properties(FuelType::Charcoal).unwrap();
        assert_eq!(charcoal.fuel_type, FuelType::Charcoal);
        assert!(charcoal.heating_value_j_per_kg > 0.0);
        assert!(charcoal.carbon_content > 0.0 && charcoal.carbon_content <= 1.0);
        assert!(charcoal.sulfur_content < 0.01);
        assert!(charcoal.ash_content < 0.1);
    }

    #[test]
    fn test_fuel_properties_coal() {
        let system = FuelSystem::new();
        let coal = system.get_fuel_properties(FuelType::Coal).unwrap();
        assert_eq!(coal.fuel_type, FuelType::Coal);
        assert!(coal.heating_value_j_per_kg > 0.0);
        assert!(coal.sulfur_content > 0.01);
    }

    #[test]
    fn test_fuel_properties_coke() {
        let system = FuelSystem::new();
        let coke = system.get_fuel_properties(FuelType::Coke).unwrap();
        assert_eq!(coke.fuel_type, FuelType::Coke);
        assert!(coke.carbon_content > 0.9);
        assert!(coke.sulfur_content < 0.02);
    }

    #[test]
    fn test_fuel_properties_wood() {
        let system = FuelSystem::new();
        let wood = system.get_fuel_properties(FuelType::Wood).unwrap();
        assert_eq!(wood.fuel_type, FuelType::Wood);
        assert!(wood.heating_value_j_per_kg < 20_000_000.0);
        assert!(wood.volatile_matter > 0.5);
    }

    #[test]
    fn test_heating_value_ordering() {
        let system = FuelSystem::new();
        let charcoal = system.get_fuel_properties(FuelType::Charcoal).unwrap();
        let coal = system.get_fuel_properties(FuelType::Coal).unwrap();
        let coke = system.get_fuel_properties(FuelType::Coke).unwrap();
        let wood = system.get_fuel_properties(FuelType::Wood).unwrap();

        assert!(charcoal.heating_value_j_per_kg > coke.heating_value_j_per_kg);
        assert!(coke.heating_value_j_per_kg > coal.heating_value_j_per_kg);
        assert!(coal.heating_value_j_per_kg > wood.heating_value_j_per_kg);
    }

    #[test]
    fn test_sulfur_content_ordering() {
        let system = FuelSystem::new();
        let charcoal = system.get_fuel_properties(FuelType::Charcoal).unwrap();
        let coal = system.get_fuel_properties(FuelType::Coal).unwrap();
        let coke = system.get_fuel_properties(FuelType::Coke).unwrap();

        assert!(coal.sulfur_content > coke.sulfur_content);
        assert!(coal.sulfur_content > charcoal.sulfur_content);
        assert!(charcoal.sulfur_content < 0.005);
    }

    #[test]
    fn test_all_fuel_types_count() {
        let system = FuelSystem::new();
        assert_eq!(system.all_fuel_types().len(), 4);
    }

    #[test]
    fn test_combustion_rate_normal() {
        let comparator = FuelComparator::new();
        let rate = comparator.calculate_combustion_rate(FuelType::Charcoal, 1200.0, 0.5);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_combustion_rate_higher_temp_higher_rate() {
        let comparator = FuelComparator::new();
        let rate_low = comparator.calculate_combustion_rate(FuelType::Charcoal, 800.0, 0.5);
        let rate_high = comparator.calculate_combustion_rate(FuelType::Charcoal, 1400.0, 0.5);
        assert!(rate_high > rate_low);
    }

    #[test]
    fn test_combustion_rate_higher_airflow_higher_rate() {
        let comparator = FuelComparator::new();
        let rate_low = comparator.calculate_combustion_rate(FuelType::Coal, 1200.0, 0.001);
        let rate_high = comparator.calculate_combustion_rate(FuelType::Coal, 1200.0, 0.01);
        assert!(rate_high > rate_low);
    }

    #[test]
    fn test_combustion_rate_zero_airflow() {
        let comparator = FuelComparator::new();
        let rate = comparator.calculate_combustion_rate(FuelType::Charcoal, 1200.0, 0.0);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_combustion_rate_extreme_temp() {
        let comparator = FuelComparator::new();
        let rate_cold = comparator.calculate_combustion_rate(FuelType::Charcoal, 0.0, 0.5);
        let rate_hot = comparator.calculate_combustion_rate(FuelType::Charcoal, 2000.0, 0.5);
        assert!(rate_cold > 0.0);
        assert!(rate_hot > 0.0);
        assert!(rate_hot > rate_cold);
    }

    #[test]
    fn test_heat_release_rate_normal() {
        let comparator = FuelComparator::new();
        let heat = comparator.calculate_heat_release_rate(FuelType::Charcoal, 0.01, 0.75);
        assert!(heat > 0.0);
    }

    #[test]
    fn test_heat_release_rate_zero_burn_rate() {
        let comparator = FuelComparator::new();
        let heat = comparator.calculate_heat_release_rate(FuelType::Charcoal, 0.0, 0.75);
        assert!(approx_eq(heat, 0.0, 1e-6));
    }

    #[test]
    fn test_heat_release_rate_zero_efficiency() {
        let comparator = FuelComparator::new();
        let heat = comparator.calculate_heat_release_rate(FuelType::Charcoal, 0.01, 0.0);
        assert!(approx_eq(heat, 0.0, 1e-6));
    }

    #[test]
    fn test_heat_release_rate_proportional_to_burn_rate() {
        let comparator = FuelComparator::new();
        let heat1 = comparator.calculate_heat_release_rate(FuelType::Charcoal, 0.01, 0.75);
        let heat2 = comparator.calculate_heat_release_rate(FuelType::Charcoal, 0.02, 0.75);
        assert!(approx_eq(heat2, heat1 * 2.0, 1e-6));
    }

    #[test]
    fn test_temp_effect_normal() {
        let comparator = FuelComparator::new();
        let new_temp = comparator.calculate_temp_effect(FuelType::Charcoal, 200.0, 100.0, 0.01);
        assert!(new_temp >= 200.0);
    }

    #[test]
    fn test_temp_effect_coke_highest() {
        let comparator = FuelComparator::new();
        let temp_wood = comparator.calculate_temp_effect(FuelType::Wood, 200.0, 100.0, 0.01);
        let temp_coke = comparator.calculate_temp_effect(FuelType::Coke, 200.0, 100.0, 0.01);
        assert!(temp_coke >= temp_wood);
    }

    #[test]
    fn test_temp_effect_zero_burn_rate() {
        let comparator = FuelComparator::new();
        let new_temp = comparator.calculate_temp_effect(FuelType::Charcoal, 800.0, 0.0, 0.02);
        assert!(new_temp <= 800.0);
    }

    #[test]
    fn test_iron_quality_normal() {
        let system = FuelSystem::new();
        let q = system.calculate_iron_quality(FuelType::Charcoal, 1300.0, 0.03);
        assert!(q.purity > 0.0 && q.purity <= 1.0);
        assert!(q.hardness > 0.0 && q.hardness <= 1.0);
        assert!(q.tensile_strength > 0.0 && q.tensile_strength <= 1.0);
        assert!(q.overall_quality > 0.0 && q.overall_quality <= 1.0);
        assert!(!q.grade.is_empty());
    }

    #[test]
    fn test_iron_quality_optimal_temp_range() {
        let system = FuelSystem::new();
        let q_optimal = system.calculate_iron_quality(FuelType::Charcoal, 1300.0, 0.03);
        let q_low = system.calculate_iron_quality(FuelType::Charcoal, 600.0, 0.03);
        let q_high = system.calculate_iron_quality(FuelType::Charcoal, 1500.0, 0.03);
        assert!(q_optimal.overall_quality >= q_low.overall_quality);
        assert!(q_optimal.overall_quality >= q_high.overall_quality);
    }

    #[test]
    fn test_iron_quality_charcoal_vs_coal() {
        let system = FuelSystem::new();
        let q_charcoal = system.calculate_iron_quality(FuelType::Charcoal, 1300.0, 0.03);
        let q_coal = system.calculate_iron_quality(FuelType::Coal, 1300.0, 0.03);
        assert!(q_charcoal.sulfur_content < q_coal.sulfur_content);
        assert!(q_charcoal.purity > q_coal.purity);
    }

    #[test]
    fn test_iron_quality_co_factor() {
        let system = FuelSystem::new();
        let q_no_co = system.calculate_iron_quality(FuelType::Charcoal, 1300.0, 0.0);
        let q_with_co = system.calculate_iron_quality(FuelType::Charcoal, 1300.0, 0.05);
        assert!(q_with_co.purity >= q_no_co.purity);
    }

    #[test]
    fn test_iron_quality_boundary_temp_zero() {
        let system = FuelSystem::new();
        let q = system.calculate_iron_quality(FuelType::Charcoal, 0.0, 0.03);
        assert!(q.overall_quality > 0.0);
        assert!(q.overall_quality <= 1.0);
    }

    #[test]
    fn test_iron_quality_boundary_temp_extreme() {
        let system = FuelSystem::new();
        let q = system.calculate_iron_quality(FuelType::Charcoal, 2000.0, 0.03);
        assert!(q.overall_quality > 0.0);
        assert!(q.overall_quality <= 1.0);
    }

    #[test]
    fn test_compare_fuels_normal() {
        let comparator = FuelComparator::new();
        let request = FuelComparisonRequest {
            furnace_type: FurnaceType::HanChaogang,
            fuels: vec![FuelType::Charcoal, FuelType::Coal, FuelType::Wood],
            target_temp: 1200.0,
            duration_hours: 8.0,
            iron_ore_kg: 1000.0,
        };
        let result = comparator.compare_fuels(&request);
        assert_eq!(result.results.len(), 3);
        assert!(!result.comparison_summary.is_empty());
    }

    #[test]
    fn test_compare_fuels_single_fuel() {
        let comparator = FuelComparator::new();
        let request = FuelComparisonRequest {
            furnace_type: FurnaceType::HanChaogang,
            fuels: vec![FuelType::Charcoal],
            target_temp: 1200.0,
            duration_hours: 8.0,
            iron_ore_kg: 1000.0,
        };
        let result = comparator.compare_fuels(&request);
        assert_eq!(result.results.len(), 1);
        assert!(result.comparison_summary.contains("至少两种"));
    }

    #[test]
    fn test_compare_fuels_all_fuels() {
        let comparator = FuelComparator::new();
        let request = FuelComparisonRequest {
            furnace_type: FurnaceType::MingBlast,
            fuels: vec![FuelType::Charcoal, FuelType::Coal, FuelType::Coke, FuelType::Wood],
            target_temp: 1400.0,
            duration_hours: 12.0,
            iron_ore_kg: 2000.0,
        };
        let result = comparator.compare_fuels(&request);
        assert_eq!(result.results.len(), 4);
        for item in &result.results {
            assert!(item.iron_output_kg > 0.0);
            assert!(item.fuel_consumed_kg > 0.0);
            assert!(item.energy_efficiency > 0.0);
        }
    }

    #[test]
    fn test_compare_fuels_boundary_zero_ore() {
        let comparator = FuelComparator::new();
        let request = FuelComparisonRequest {
            furnace_type: FurnaceType::HanChaogang,
            fuels: vec![FuelType::Charcoal, FuelType::Coal],
            target_temp: 1200.0,
            duration_hours: 8.0,
            iron_ore_kg: 0.0,
        };
        let result = comparator.compare_fuels(&request);
        for item in &result.results {
            assert!(item.iron_output_kg >= 0.0);
        }
    }

    #[test]
    fn test_compare_fuels_boundary_zero_duration() {
        let comparator = FuelComparator::new();
        let request = FuelComparisonRequest {
            furnace_type: FurnaceType::HanChaogang,
            fuels: vec![FuelType::Charcoal, FuelType::Coal],
            target_temp: 1200.0,
            duration_hours: 0.0,
            iron_ore_kg: 1000.0,
        };
        let result = comparator.compare_fuels(&request);
        for item in &result.results {
            assert!(item.fuel_consumed_kg >= 0.0);
        }
    }

    #[test]
    fn test_compare_fuels_ming_blast_higher_efficiency() {
        let comparator = FuelComparator::new();
        let req_han = FuelComparisonRequest {
            furnace_type: FurnaceType::HanChaogang,
            fuels: vec![FuelType::Charcoal],
            target_temp: 1300.0,
            duration_hours: 8.0,
            iron_ore_kg: 1000.0,
        };
        let req_ming = FuelComparisonRequest {
            furnace_type: FurnaceType::MingBlast,
            fuels: vec![FuelType::Charcoal],
            target_temp: 1300.0,
            duration_hours: 8.0,
            iron_ore_kg: 1000.0,
        };
        let res_han = comparator.compare_fuels(&req_han);
        let res_ming = comparator.compare_fuels(&req_ming);
        assert!(res_ming.results[0].energy_efficiency > res_han.results[0].energy_efficiency);
    }

    #[test]
    fn test_slag_composition_normal() {
        let system = FuelSystem::new();
        let slag = system.calculate_slag_composition(FuelType::Charcoal, "hematite", 1300.0);
        let total = slag.total();
        assert!((total - 1.0).abs() < 0.05);
        assert!(slag.sio2 > 0.0);
        assert!(slag.feo >= 0.0);
    }

    #[test]
    fn test_slag_composition_coal_higher_ash() {
        let system = FuelSystem::new();
        let slag_charcoal = system.calculate_slag_composition(FuelType::Charcoal, "hematite", 1300.0);
        let slag_coal = system.calculate_slag_composition(FuelType::Coal, "hematite", 1300.0);
        assert!(slag_coal.s_content > slag_charcoal.s_content);
    }

    #[test]
    fn test_slag_composition_ore_types() {
        let system = FuelSystem::new();
        for ore in &["hematite", "magnetite", "limonite", "siderite", "unknown"] {
            let slag = system.calculate_slag_composition(FuelType::Charcoal, ore, 1300.0);
            let total = slag.total();
            assert!((total - 1.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_temp_effect_higher_burn_rate_higher_temp() {
        let comparator = FuelComparator::new();
        let temp_low = comparator.calculate_temp_effect(FuelType::Charcoal, 800.0, 5.0, 0.02);
        let temp_high = comparator.calculate_temp_effect(FuelType::Charcoal, 800.0, 20.0, 0.02);
        assert!(temp_high >= temp_low);
    }

    #[test]
    fn test_fuel_comparator_new() {
        let comparator = FuelComparator::new();
        let props = comparator.get_all_fuel_properties();
        assert_eq!(props.len(), 4);
    }

    #[test]
    fn test_fuel_comparator_get_all_fuel_properties() {
        let comparator = FuelComparator::new();
        let props = comparator.get_all_fuel_properties();
        assert_eq!(props.len(), 4);
        for p in props {
            assert!(p.heating_value_j_per_kg > 0.0);
        }
    }

    #[test]
    fn test_fuel_comparator_get_iron_quality() {
        let comparator = FuelComparator::new();
        let q = comparator.get_iron_quality(FuelType::Charcoal, 1300.0, 0.03, "hematite");
        assert!(q.overall_quality > 0.0 && q.overall_quality <= 1.0);
    }

    #[test]
    fn test_fuel_comparator_get_slag_composition() {
        let comparator = FuelComparator::new();
        let slag = comparator.get_slag_composition(FuelType::Charcoal, "hematite");
        let total = slag.total();
        assert!((total - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_fuel_comparator_get_literature_source() {
        let comparator = FuelComparator::new();
        let source = comparator.get_literature_source(FuelType::Charcoal);
        assert!(source.is_some());
        let source = source.unwrap();
        assert!(!source.literature_reference.is_empty());
        assert!(source.measurement_year > 0);
        assert!(source.value_confidence > 0.0 && source.value_confidence <= 1.0);
    }

    #[test]
    fn test_fuel_comparator_get_literature_source_all_fuels() {
        let comparator = FuelComparator::new();
        for fuel in &[FuelType::Charcoal, FuelType::Coal, FuelType::Coke, FuelType::Wood] {
            let source = comparator.get_literature_source(*fuel);
            assert!(source.is_some());
        }
    }

    #[test]
    fn test_fuel_comparator_default() {
        let comparator = FuelComparator::default();
        let props = comparator.get_all_fuel_properties();
        assert_eq!(props.len(), 4);
    }
}
