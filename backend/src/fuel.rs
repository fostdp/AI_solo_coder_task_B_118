use std::collections::HashMap;

use tracing::{debug, info};

use crate::models::{
    FurnaceType, FuelComparisonItem, FuelComparisonRequest, FuelComparisonResult, FuelProperties,
    FuelType, IronQualityMetrics,
};

pub struct FuelSystem {
    fuel_properties: HashMap<FuelType, FuelProperties>,
    default_furnace_type: FurnaceType,
}

impl FuelSystem {
    pub fn new() -> Self {
        let mut fuel_properties = HashMap::new();
        fuel_properties.insert(FuelType::Charcoal, FuelProperties::charcoal());
        fuel_properties.insert(FuelType::Coal, FuelProperties::coal());
        fuel_properties.insert(FuelType::Coke, FuelProperties::coke());
        fuel_properties.insert(FuelType::Wood, FuelProperties::wood());

        Self {
            fuel_properties,
            default_furnace_type: FurnaceType::HanChaogang,
        }
    }

    pub fn get_fuel_properties(&self, fuel_type: FuelType) -> Option<&FuelProperties> {
        self.fuel_properties.get(&fuel_type)
    }

    pub fn all_fuel_types(&self) -> Vec<FuelType> {
        self.fuel_properties.keys().copied().collect()
    }

    pub fn all_fuel_properties(&self) -> Vec<&FuelProperties> {
        self.fuel_properties.values().collect()
    }

    pub fn calculate_combustion_rate(
        &self,
        fuel_type: FuelType,
        temp_c: f64,
        air_flow_m3_per_s: f64,
    ) -> f64 {
        let props = match self.fuel_properties.get(&fuel_type) {
            Some(p) => p,
            None => return 0.0,
        };

        let temp_factor = 1.0 + (temp_c - 800.0) / 1000.0;
        let temp_factor = temp_factor.max(0.3).min(2.0);

        let air_factor = (air_flow_m3_per_s * 3600.0 / 100.0).min(2.0);
        let air_factor = air_factor.max(0.1);

        props.burn_rate_factor * temp_factor * air_factor
    }

    pub fn calculate_heat_release_rate(
        &self,
        fuel_type: FuelType,
        fuel_burn_rate_kg_per_s: f64,
        efficiency: f64,
    ) -> f64 {
        let props = match self.fuel_properties.get(&fuel_type) {
            Some(p) => p,
            None => return 0.0,
        };

        props.heating_value_j_per_kg * fuel_burn_rate_kg_per_s * efficiency
    }

    pub fn calculate_temp_effect(
        &self,
        fuel_type: FuelType,
        base_temp: f64,
        fuel_burn_rate_kg_per_h: f64,
        heat_loss_coeff: f64,
    ) -> f64 {
        let props = match self.fuel_properties.get(&fuel_type) {
            Some(p) => p,
            None => return base_temp,
        };

        let heat_input = props.heating_value_j_per_kg * fuel_burn_rate_kg_per_h / 3600.0;
        let efficiency = 0.75 * props.burn_rate_factor;
        let net_heat = heat_input * efficiency;

        let temp_rise = net_heat / (heat_loss_coeff * 1000.0 + 500.0);
        let equilibrium_temp = 25.0 + temp_rise;

        let temp_diff = equilibrium_temp - base_temp;
        base_temp + temp_diff * 0.05
    }

    pub fn calculate_iron_quality(
        &self,
        fuel_type: FuelType,
        temp_c: f64,
        co_level: f64,
    ) -> IronQualityMetrics {
        let props = match self.fuel_properties.get(&fuel_type) {
            Some(p) => p,
            None => {
                return IronQualityMetrics {
                    purity: 0.7,
                    hardness: 0.5,
                    tensile_strength: 0.5,
                    carbon_content: 0.03,
                    sulfur_content: 0.005,
                    phosphorus_content: 0.002,
                    grain_size: 0.5,
                    overall_quality: 0.6,
                    grade: IronQualityMetrics::grade_from_score(0.6).to_string(),
                }
            }
        };

        let temp_factor = if temp_c >= 1200.0 && temp_c <= 1400.0 {
            1.0
        } else if temp_c < 1200.0 {
            (temp_c - 800.0) / 400.0
        } else {
            1.0 - (temp_c - 1400.0) / 400.0
        }
        .max(0.3);

        let impurity_penalty = props.impurity_level * 0.5;
        let sulfur_content = props.sulfur_content;
        let carbon_content = props.carbon_content * 0.02;

        let co_factor = if co_level > 0.0 {
            (co_level / 0.05).min(1.0)
        } else {
            0.1
        };

        let reduction_quality = 0.5 + co_factor * 0.5;

        let purity = (1.0 - impurity_penalty) * temp_factor * reduction_quality;
        let hardness = 0.4 + carbon_content * 10.0 + temp_factor * 0.3;
        let tensile_strength = purity * 0.7 + hardness * 0.3;

        let overall = purity * 0.4 + hardness * 0.2 + tensile_strength * 0.2
            + (1.0 - sulfur_content / 0.05) * 0.1
            + temp_factor * 0.1;

        IronQualityMetrics {
            purity: purity.max(0.0).min(1.0),
            hardness: hardness.max(0.0).min(1.0),
            tensile_strength: tensile_strength.max(0.0).min(1.0),
            carbon_content: carbon_content.max(0.0).min(0.1),
            sulfur_content: sulfur_content.max(0.0).min(0.1),
            phosphorus_content: props.ash_content * 0.01,
            grain_size: 0.5 + temp_factor * 0.3 - impurity_penalty * 0.5,
            overall_quality: overall.max(0.0).min(1.0),
            grade: IronQualityMetrics::grade_from_score(overall.max(0.0).min(1.0)).to_string(),
        }
    }

    pub fn compare_fuels(&self, request: &FuelComparisonRequest) -> FuelComparisonResult {
        debug!(
            "Comparing {} fuels for {:?}",
            request.fuels.len(),
            request.furnace_type
        );

        let mut results = Vec::new();

        for &fuel_type in &request.fuels {
            let props = match self.fuel_properties.get(&fuel_type) {
                Some(p) => p,
                None => continue,
            };

            let item = self.simulate_fuel_performance(
                fuel_type,
                props,
                request.target_temp,
                request.duration_hours,
                request.iron_ore_kg,
                request.furnace_type,
            );
            results.push(item);
        }

        let recommendation = self.select_recommendation(&results, &request.fuels);

        let summary = self.generate_comparison_summary(&results, recommendation);

        FuelComparisonResult {
            request: request.clone(),
            results,
            recommendation,
            comparison_summary: summary,
        }
    }

    fn simulate_fuel_performance(
        &self,
        fuel_type: FuelType,
        props: &FuelProperties,
        target_temp: f64,
        duration_hours: f64,
        iron_ore_kg: f64,
        furnace_type: FurnaceType,
    ) -> FuelComparisonItem {
        let furnace_efficiency_factor = match furnace_type {
            FurnaceType::HanChaogang => 0.75,
            FurnaceType::MingBlast => 0.85,
        };

        let heat_required_per_hour = target_temp * 5000.0 / furnace_efficiency_factor;
        let fuel_kg_per_hour =
            heat_required_per_hour / (props.heating_value_j_per_kg * 0.7);

        let total_fuel = fuel_kg_per_hour * duration_hours;

        let temp_stability = if props.burn_rate_factor >= 0.9 {
            0.85
        } else if props.burn_rate_factor >= 0.8 {
            0.75
        } else {
            0.6
        };

        let reduction_ratio = 0.6 + furnace_efficiency_factor * 0.1;
        let iron_output = iron_ore_kg * reduction_ratio * (1.0 - props.impurity_level * 0.3);

        let quality = self.calculate_iron_quality(fuel_type, target_temp, 0.03);

        let slag_amount = total_fuel * props.ash_content + iron_ore_kg * 0.15;

        let heating_time = if props.flame_temp > target_temp {
            (target_temp - 25.0) / (props.flame_temp * 0.15)
        } else {
            duration_hours
        }
        .max(0.5);

        FuelComparisonItem {
            fuel_type,
            fuel_name: fuel_type.display_name().to_string(),
            avg_temp: target_temp * (0.9 + temp_stability * 0.1),
            max_temp: props.flame_temp.min(target_temp * 1.2),
            temp_stability,
            iron_output_kg: iron_output,
            iron_quality: quality.overall_quality,
            sulfur_content: props.sulfur_content,
            fuel_consumed_kg: total_fuel,
            fuel_cost: total_fuel * props.cost_per_kg,
            energy_efficiency: furnace_efficiency_factor * props.burn_rate_factor,
            slag_amount,
            heating_time_hours: heating_time,
        }
    }

    fn select_recommendation(
        &self,
        results: &[FuelComparisonItem],
        _requested_fuels: &[FuelType],
    ) -> FuelType {
        if results.is_empty() {
            return FuelType::Charcoal;
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_fuel = FuelType::Charcoal;

        for item in results {
            let quality_score = item.iron_quality * 40.0;
            let efficiency_score = item.energy_efficiency * 20.0;
            let cost_score = (1.0 / item.fuel_cost.max(0.1)) * 100.0;
            let output_score = (item.iron_output_kg / 1000.0).min(1.0) * 20.0;

            let total = quality_score + efficiency_score + cost_score + output_score;

            if total > best_score {
                best_score = total;
                best_fuel = item.fuel_type;
            }
        }

        best_fuel
    }

    fn generate_comparison_summary(
        &self,
        results: &[FuelComparisonItem],
        recommendation: FuelType,
    ) -> String {
        if results.len() < 2 {
            return "请选择至少两种燃料进行对比".to_string();
        }

        let mut summary = String::new();

        let best_quality = results
            .iter()
            .max_by_key(|r| (r.iron_quality * 1000.0) as i32)
            .unwrap();
        let lowest_cost = results
            .iter()
            .min_by_key(|r| (r.fuel_cost * 100.0) as i32)
            .unwrap();
        let highest_output = results
            .iter()
            .max_by_key(|r| (r.iron_output_kg * 100.0) as i32)
            .unwrap();

        summary.push_str(&format!(
            "综合推荐使用{}。",
            recommendation.display_name()
        ));
        summary.push_str(&format!(
            " {}的铁水质量最高({:.0}分)，",
            best_quality.fuel_name,
            best_quality.iron_quality * 100.0
        ));
        summary.push_str(&format!(
            " {}的生产成本最低(约{:.0}元)，",
            lowest_cost.fuel_name, lowest_cost.fuel_cost
        ));
        summary.push_str(&format!(
            " {}的产铁量最大({:.0}kg)。",
            highest_output.fuel_name, highest_output.iron_output_kg
        ));

        summary
    }

    pub fn calculate_slag_composition(
        &self,
        fuel_type: FuelType,
        ore_type: &str,
        temp_c: f64,
    ) -> crate::models::SlagComposition {
        let props = match self.fuel_properties.get(&fuel_type) {
            Some(p) => p,
            None => return crate::models::SlagComposition::default(),
        };

        let base_ash = props.ash_content;

        let (ore_sio2, ore_al2o3, ore_cao, ore_feo) = match ore_type {
            "hematite" => (0.10, 0.03, 0.02, 0.85),
            "magnetite" => (0.08, 0.02, 0.01, 0.90),
            "limonite" => (0.12, 0.05, 0.03, 0.75),
            "siderite" => (0.05, 0.02, 0.05, 0.70),
            _ => (0.15, 0.05, 0.03, 0.70),
        };

        let ash_ratio = base_ash * 0.4;
        let ore_gangue = 1.0 - ore_feo;

        let total_gangue = ash_ratio + ore_gangue;

        let sio2 = (base_ash * 0.4 + ore_sio2 * ore_gangue) / total_gangue;
        let al2o3 = (base_ash * 0.15 + ore_al2o3 * ore_gangue) / total_gangue;
        let cao = (base_ash * 0.08 + ore_cao * ore_gangue) / total_gangue;
        let mgo = (base_ash * 0.05 + 0.01 * ore_gangue) / total_gangue;
        let feo = (base_ash * 0.1 + ore_feo * 0.05) / total_gangue;

        let temp_factor = ((temp_c - 1000.0) / 500.0).max(0.0).min(1.0);
        let feo_reduction = feo * temp_factor * 0.3;

        crate::models::SlagComposition {
            sio2: sio2.max(0.0).min(1.0),
            al2o3: al2o3.max(0.0).min(1.0),
            cao: cao.max(0.0).min(1.0),
            mgo: mgo.max(0.0).min(1.0),
            feo: (feo - feo_reduction).max(0.0).min(1.0),
            mno: 0.02,
            p2o5: props.ash_content * 0.01,
            s_content: props.sulfur_content * 0.5,
            tio2: 0.01,
            v2o5: 0.003,
            cr2o3: 0.002,
            ni_o: 0.001,
        }
        .normalize()
    }
}

impl Default for FuelSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuel_properties() {
        let system = FuelSystem::new();

        let charcoal = system.get_fuel_properties(FuelType::Charcoal).unwrap();
        assert_eq!(charcoal.fuel_type, FuelType::Charcoal);
        assert!(charcoal.heating_value_j_per_kg > 0.0);

        let coal = system.get_fuel_properties(FuelType::Coal).unwrap();
        assert_eq!(coal.fuel_type, FuelType::Coal);
        assert!(coal.sulfur_content > charcoal.sulfur_content);
    }

    #[test]
    fn test_iron_quality_calculation() {
        let system = FuelSystem::new();

        let quality_charcoal =
            system.calculate_iron_quality(FuelType::Charcoal, 1300.0, 0.03);
        let quality_coal = system.calculate_iron_quality(FuelType::Coal, 1300.0, 0.03);

        assert!(quality_charcoal.overall_quality > 0.0);
        assert!(quality_charcoal.overall_quality <= 1.0);
        assert!(quality_charcoal.sulfur_content < quality_coal.sulfur_content);
    }

    #[test]
    fn test_fuel_comparison() {
        let system = FuelSystem::new();

        let request = FuelComparisonRequest {
            furnace_type: FurnaceType::HanChaogang,
            fuels: vec![FuelType::Charcoal, FuelType::Coal, FuelType::Wood],
            target_temp: 1200.0,
            duration_hours: 8.0,
            iron_ore_kg: 1000.0,
        };

        let result = system.compare_fuels(&request);
        assert_eq!(result.results.len(), 3);
        assert!(!result.comparison_summary.is_empty());
    }
}
