use tracing::{debug, info};

use crate::models::{
    OreSourceCandidate, ProcessInference, SlagAnalysisRequest, SlagAnalysisResult,
    SlagComposition,
};

const ORE_SOURCE_DATABASE: &[(&str, &str, f64, f64, f64, f64, &[&str], &str)] = &[
    (
        "河北邯郸",
        "磁铁矿",
        0.12,
        0.03,
        0.003,
        0.001,
        &["Ti", "V"],
        "典型的华北地台型铁矿床，以磁铁矿为主，钛钒含量较高。",
    ),
    (
        "湖北大冶",
        "赤铁矿",
        0.20,
        0.06,
        0.008,
        0.002,
        &["Cu", "Co"],
        "矽卡岩型铁矿，伴生铜钴等有色金属。",
    ),
    (
        "四川攀枝花",
        "钒钛磁铁矿",
        0.25,
        0.10,
        0.030,
        0.015,
        &["Ti", "V", "Cr"],
        "辉长岩型钒钛磁铁矿，钛钒铬含量极高，是西南地区特色矿种。",
    ),
    (
        "辽宁鞍山",
        "变质铁矿",
        0.45,
        0.08,
        0.002,
        0.001,
        &["Si", "Al"],
        "前寒武纪变质铁矿床，硅铝含量高，属于条带状铁建造(BIF)。",
    ),
    (
        "安徽马鞍山",
        "火山岩型铁矿",
        0.18,
        0.05,
        0.005,
        0.008,
        &["P", "S"],
        "陆相火山岩型铁矿，磷硫含量偏高。",
    ),
    (
        "云南个旧",
        "锡铁共生矿",
        0.15,
        0.04,
        0.002,
        0.003,
        &["Sn", "Cu", "Pb"],
        "锡石硫化物型矿床，铁锡铜铅锌多金属共生。",
    ),
    (
        "甘肃镜铁山",
        "镜铁矿",
        0.35,
        0.07,
        0.004,
        0.002,
        &["Ba", "F"],
        "祁连山型沉积变质铁矿，重晶石和萤石含量较高。",
    ),
    (
        "新疆哈密",
        "岩浆型铁矿",
        0.28,
        0.12,
        0.020,
        0.010,
        &["Ti", "V", "Ni"],
        "基性-超基性岩型铁矿，钛钒镍等元素含量高。",
    ),
];

pub struct SlagAnalysisSystem {
    ore_sources: Vec<OreSourceData>,
}

struct OreSourceData {
    region: String,
    ore_type: String,
    sio2: f64,
    al2o3: f64,
    tio2: f64,
    v2o5: f64,
    characteristic_elements: Vec<String>,
    description: String,
}

impl SlagAnalysisSystem {
    pub fn new() -> Self {
        let ore_sources = ORE_SOURCE_DATABASE
            .iter()
            .map(|(region, ore_type, sio2, al2o3, tio2, v2o5, elements, desc)| {
                OreSourceData {
                    region: region.to_string(),
                    ore_type: ore_type.to_string(),
                    sio2: *sio2,
                    al2o3: *al2o3,
                    tio2: *tio2,
                    v2o5: *v2o5,
                    characteristic_elements: elements.iter().map(|s| s.to_string()).collect(),
                    description: desc.to_string(),
                }
            })
            .collect();

        Self { ore_sources }
    }

    pub fn analyze(&self, request: &SlagAnalysisRequest) -> SlagAnalysisResult {
        debug!("Analyzing slag composition");

        let composition = request.composition.normalize();
        let basicity = composition.basicity();
        let quaternary_basicity = composition.quaternary_basicity();

        let melting_point = self.estimate_melting_point(&composition, basicity);
        let viscosity = self.estimate_viscosity(&composition, melting_point, 1400.0);

        let slag_type = self.classify_slag_type(basicity, quaternary_basicity);

        let process_inference = self.infer_process(&composition, basicity, request.furnace_type);

        let ore_candidates = self.match_ore_sources(&composition);

        let iron_quality = self.estimate_iron_quality(&composition);

        let summary = self.generate_analysis_summary(
            &composition,
            basicity,
            &process_inference,
            &ore_candidates,
        );

        SlagAnalysisResult {
            composition,
            basicity,
            quaternary_basicity,
            melting_point_c: melting_point,
            viscosity_pa_s: viscosity,
            slag_type,
            process_inference,
            ore_source_candidates: ore_candidates,
            iron_quality_estimate: iron_quality,
            analysis_summary: summary,
        }
    }

    fn estimate_melting_point(&self, composition: &SlagComposition, basicity: f64) -> f64 {
        let base_temp = 1150.0;

        let sio2_effect = composition.sio2 * -800.0;
        let cao_effect = composition.cao * 600.0;
        let mgo_effect = composition.mgo * 400.0;
        let al2o3_effect = composition.al2o3 * 200.0;
        let feo_effect = composition.feo * -300.0;

        let basicity_factor = if basicity > 1.0 {
            (basicity - 1.0) * 100.0
        } else {
            (1.0 - basicity) * 150.0
        };

        let melting_point = base_temp + sio2_effect + cao_effect + mgo_effect
            + al2o3_effect + feo_effect - basicity_factor;

        melting_point.max(900.0).min(1600.0)
    }

    fn estimate_viscosity(
        &self,
        _composition: &SlagComposition,
        melting_point: f64,
        operating_temp: f64,
    ) -> f64 {
        let temp_diff = operating_temp - melting_point;

        if temp_diff <= 0.0 {
            return 100.0;
        }

        let base_viscosity = 0.5;
        let viscosity = base_viscosity * (-temp_diff / 100.0).exp() * 5.0;

        viscosity.max(0.01).min(50.0)
    }

    fn classify_slag_type(&self, basicity: f64, _quaternary_basicity: f64) -> String {
        if basicity < 0.5 {
            "酸性渣 (Acidic Slag)".to_string()
        } else if basicity < 0.8 {
            "弱酸性渣 (Sub-acidic Slag)".to_string()
        } else if basicity < 1.2 {
            "中性渣 (Neutral Slag)".to_string()
        } else if basicity < 1.5 {
            "弱碱性渣 (Sub-basic Slag)".to_string()
        } else if basicity < 2.0 {
            "碱性渣 (Basic Slag)".to_string()
        } else {
            "高碱性渣 (High-basic Slag)".to_string()
        }
    }

    fn infer_process(
        &self,
        composition: &SlagComposition,
        basicity: f64,
        furnace_type: Option<crate::models::FurnaceType>,
    ) -> ProcessInference {
        let mut evidence = Vec::new();
        let mut confidence: f64 = 0.5;

        let estimated_temp = 1200.0 + (basicity - 1.0) * 200.0 + composition.feo * -500.0;
        let estimated_temp = estimated_temp.max(900.0).min(1600.0);
        let temp_conf = 0.6 + (1.0 - (composition.feo - 0.05).abs()) * 0.3;

        let reduction_level = if composition.feo > 0.15 {
            evidence.push("FeO含量高，说明还原程度较低".to_string());
            0.3
        } else if composition.feo > 0.08 {
            evidence.push("FeO含量中等，还原程度一般".to_string());
            0.6
        } else {
            evidence.push("FeO含量低，说明还原充分".to_string());
            0.85
        };

        let reduction_atmosphere = if composition.s_content > 0.01 {
            evidence.push("硫含量较高，可能使用了高硫燃料".to_string());
            "弱还原性".to_string()
        } else if composition.s_content > 0.003 {
            "中等还原性".to_string()
        } else {
            evidence.push("硫含量低，可能使用了木炭或优质燃料".to_string());
            "强还原性".to_string()
        };

        let process_type = if basicity > 1.5 {
            evidence.push("高碱度渣，可能使用了石灰石作熔剂".to_string());
            confidence += 0.1;
            "碱性熔炼法".to_string()
        } else if basicity < 0.6 {
            evidence.push("酸性渣，可能是未加熔剂的自然熔炼".to_string());
            confidence += 0.1;
            "酸性熔炼法".to_string()
        } else {
            "中性熔炼法".to_string()
        };

        let smelting_period = match furnace_type {
            Some(crate::models::FurnaceType::HanChaogang) => {
                evidence.push("对应汉代炒钢炉，属于西汉至东汉时期".to_string());
                "汉代 (公元前206年 - 公元220年)".to_string()
            }
            Some(crate::models::FurnaceType::MingBlast) => {
                evidence.push("对应明代高炉，属于明清时期".to_string());
                "明代 (1368年 - 1644年)".to_string()
            }
            None => {
                if basicity > 1.2 {
                    evidence.push("较高碱度暗示使用了熔剂，可能是较晚时期".to_string());
                    "宋代以后 (960年以后)".to_string()
                } else {
                    "汉代至唐宋 (公元前206年 - 1279年)".to_string()
                }
            }
        };

        let fuel_hint = if composition.s_content > 0.015 {
            evidence.push("高硫特征，推测使用煤炭或焦炭".to_string());
            confidence += 0.05;
            "煤炭/焦炭".to_string()
        } else if composition.s_content > 0.005 {
            evidence.push("硫含量中等，可能是木炭或混合燃料".to_string());
            "木炭为主，可能混用少量煤".to_string()
        } else {
            evidence.push("低硫特征，推测使用木炭".to_string());
            confidence += 0.05;
            "木炭".to_string()
        };

        confidence = confidence.min(0.95).max(0.2);

        ProcessInference {
            estimated_temp_c: estimated_temp,
            temp_confidence: temp_conf,
            reduction_atmosphere,
            reduction_level,
            smelting_period,
            process_type,
            fuel_type_hint: fuel_hint,
            confidence,
            evidence,
        }
    }

    fn match_ore_sources(&self, composition: &SlagComposition) -> Vec<OreSourceCandidate> {
        let mut candidates = Vec::new();

        for source in &self.ore_sources {
            let sio2_diff = (composition.sio2 - source.sio2).abs();
            let al2o3_diff = (composition.al2o3 - source.al2o3).abs();
            let tio2_diff = (composition.tio2 - source.tio2).abs();
            let v2o5_diff = (composition.v2o5 - source.v2o5).abs();

            let sio2_score = 1.0 - (sio2_diff / 0.5).min(1.0);
            let al2o3_score = 1.0 - (al2o3_diff / 0.3).min(1.0);
            let tio2_score = 1.0 - (tio2_diff / 0.05).min(1.0);
            let v2o5_score = 1.0 - (v2o5_diff / 0.02).min(1.0);

            let total_score = sio2_score * 0.3
                + al2o3_score * 0.2
                + tio2_score * 0.25
                + v2o5_score * 0.25;

            if total_score > 0.4 {
                candidates.push(OreSourceCandidate {
                    region: source.region.clone(),
                    ore_type: source.ore_type.clone(),
                    match_score: total_score,
                    characteristic_elements: source.characteristic_elements.clone(),
                    description: source.description.clone(),
                });
            }
        }

        candidates.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());
        candidates.truncate(5);

        candidates
    }

    fn estimate_iron_quality(&self, composition: &SlagComposition) -> f64 {
        let feo_factor = 1.0 - (composition.feo / 0.2).min(1.0);
        let s_factor = 1.0 - (composition.s_content / 0.02).min(1.0);
        let p_factor = 1.0 - (composition.p2o5 / 0.02).min(1.0);

        let quality = feo_factor * 0.4 + s_factor * 0.35 + p_factor * 0.25;

        quality.max(0.2).min(0.98)
    }

    fn generate_analysis_summary(
        &self,
        composition: &SlagComposition,
        basicity: f64,
        process: &ProcessInference,
        ores: &[OreSourceCandidate],
    ) -> String {
        let mut summary = String::new();

        summary.push_str(&format!(
            "该炉渣碱度为{:.2}，{}。",
            basicity,
            if basicity > 1.0 { "偏碱性" } else { "偏酸性" }
        ));

        summary.push_str(&format!(
            "主要成分：SiO₂ {:.1}%、Al₂O₃ {:.1}%、CaO {:.1}%、FeO {:.1}%。",
            composition.sio2 * 100.0,
            composition.al2o3 * 100.0,
            composition.cao * 100.0,
            composition.feo * 100.0
        ));

        summary.push_str(&format!(
            "推测冶炼温度约{:.0}°C，{}，{}。",
            process.estimated_temp_c,
            process.reduction_atmosphere.clone(),
            process.fuel_type_hint.clone()
        ));

        if let Some(best_ore) = ores.first() {
            summary.push_str(&format!(
                "矿石来源最可能为{}的{}（匹配度{:.0}%）。",
                best_ore.region,
                best_ore.ore_type,
                best_ore.match_score * 100.0
            ));
        }

        summary
    }

    pub fn all_ore_sources(&self) -> Vec<OreSourceCandidate> {
        self.ore_sources
            .iter()
            .map(|s| OreSourceCandidate {
                region: s.region.clone(),
                ore_type: s.ore_type.clone(),
                match_score: 0.0,
                characteristic_elements: s.characteristic_elements.clone(),
                description: s.description.clone(),
            })
            .collect()
    }

    pub fn generate_slag_sample(
        &self,
        ore_source: &str,
        fuel_type: crate::models::FuelType,
        temp_c: f64,
        reduction_level: f64,
    ) -> SlagComposition {
        let ore = self
            .ore_sources
            .iter()
            .find(|s| s.region == ore_source)
            .unwrap_or(&self.ore_sources[0]);

        let fuel_props = crate::models::FuelProperties::get(fuel_type);

        let ore_gangue_sio2 = ore.sio2;
        let ore_gangue_al2o3 = ore.al2o3;
        let ore_gangue_cao = 0.02;
        let ore_gangue_mgo = 0.01;

        let fuel_ash_sio2 = fuel_props.ash_content * 0.4;
        let fuel_ash_al2o3 = fuel_props.ash_content * 0.15;
        let fuel_ash_cao = fuel_props.ash_content * 0.08;
        let fuel_ash_mgo = fuel_props.ash_content * 0.05;

        let total_gangue = ore_gangue_sio2
            + ore_gangue_al2o3
            + ore_gangue_cao
            + ore_gangue_mgo
            + fuel_ash_sio2
            + fuel_ash_al2o3
            + fuel_ash_cao
            + fuel_ash_mgo;

        let feo_base = 0.08;
        let feo = feo_base * (1.0 - reduction_level * 0.6);

        let mut composition = SlagComposition {
            sio2: (ore_gangue_sio2 + fuel_ash_sio2) / total_gangue * 0.7,
            al2o3: (ore_gangue_al2o3 + fuel_ash_al2o3) / total_gangue * 0.7,
            cao: (ore_gangue_cao + fuel_ash_cao) / total_gangue * 0.7,
            mgo: (ore_gangue_mgo + fuel_ash_mgo) / total_gangue * 0.7,
            feo,
            mno: 0.02,
            p2o5: 0.01,
            s_content: fuel_props.sulfur_content * 0.5,
            tio2: ore.tio2,
            v2o5: ore.v2o5,
            cr2o3: 0.002,
            ni_o: 0.001,
        };

        let _ = temp_c;
        composition = composition.normalize();

        composition
    }
}

impl Default for SlagAnalysisSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FurnaceType;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    fn make_composition(sio2: f64, cao: f64, mgo: f64, al2o3: f64, feo: f64,
                        mno: f64, p2o5: f64, s: f64, tio2: f64, v2o5: f64,
                        cr2o3: f64, ni_o: f64) -> SlagComposition {
        SlagComposition {
            sio2, cao, mgo, al2o3, feo, mno, p2o5, s_content: s,
            tio2, v2o5, cr2o3, ni_o,
        }
    }

    #[test]
    fn test_basicity_neutral() {
        let comp = make_composition(0.35, 0.35, 0.05, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001);
        let normalized = comp.normalize();
        let b = normalized.basicity();
        assert!(approx_eq(b, 1.0, 0.05), "basicity should be ~1.0, got {}", b);
    }

    #[test]
    fn test_basicity_acidic() {
        let comp = make_composition(0.50, 0.10, 0.03, 0.10, 0.15, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001);
        let normalized = comp.normalize();
        let b = normalized.basicity();
        assert!(b < 1.0, "acidic slag should have basicity < 1.0, got {}", b);
    }

    #[test]
    fn test_basicity_basic() {
        let comp = make_composition(0.15, 0.40, 0.10, 0.05, 0.10, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001);
        let normalized = comp.normalize();
        let b = normalized.basicity();
        assert!(b > 1.0, "basic slag should have basicity > 1.0, got {}", b);
    }

    #[test]
    fn test_basicity_zero_sio2() {
        let comp = make_composition(0.0, 0.35, 0.05, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001);
        let b = comp.basicity();
        assert_eq!(b, 0.0, "basicity with zero SiO2 should be 0.0");
    }

    #[test]
    fn test_quaternary_basicity() {
        let comp = make_composition(0.30, 0.25, 0.10, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001);
        let normalized = comp.normalize();
        let qb = normalized.quaternary_basicity();
        assert!(qb > 0.0);
        let expected = (normalized.cao + normalized.mgo) / (normalized.sio2 + normalized.al2o3);
        assert!(approx_eq(qb, expected, 1e-6));
    }

    #[test]
    fn test_quaternary_basicity_zero_denominator() {
        let comp = SlagComposition {
            sio2: 0.0, al2o3: 0.0, ..SlagComposition::default()
        };
        let qb = comp.quaternary_basicity();
        assert_eq!(qb, 0.0);
    }

    #[test]
    fn test_normalize_sum_to_one() {
        let comp = make_composition(35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01);
        let normalized = comp.normalize();
        let total = normalized.total();
        assert!(approx_eq(total, 1.0, 0.001), "normalized total should be 1.0, got {}", total);
    }

    #[test]
    fn test_normalize_zero_total() {
        let comp = make_composition(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let normalized = comp.normalize();
        assert!(approx_eq(normalized.total(), 0.0, 1e-6));
    }

    #[test]
    fn test_full_analysis_normal() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01),
            furnace_type: Some(FurnaceType::HanChaogang),
        };
        let result = system.analyze(&request);

        assert!(result.basicity > 0.0);
        assert!(!result.slag_type.is_empty());
        assert!(result.melting_point_c >= 900.0 && result.melting_point_c <= 1600.0);
        assert!(result.viscosity_pa_s >= 0.01 && result.viscosity_pa_s <= 50.0);
        assert!(result.process_inference.confidence > 0.0 && result.process_inference.confidence <= 0.95);
        assert!(!result.analysis_summary.is_empty());
    }

    #[test]
    fn test_analysis_melting_point_range() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.melting_point_c >= 900.0 && result.melting_point_c <= 1600.0);
    }

    #[test]
    fn test_analysis_viscosity_above_melting_point() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(30.0, 20.0, 10.0, 8.0, 5.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.viscosity_pa_s > 0.0);
        assert!(result.viscosity_pa_s < 100.0);
    }

    #[test]
    fn test_slag_type_classification_acidic() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(50.0, 5.0, 2.0, 15.0, 10.0, 2.0, 0.5, 0.5, 1.0, 0.05, 0.03, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.slag_type.contains("酸性") || result.slag_type.contains("Acidic"));
    }

    #[test]
    fn test_slag_type_classification_basic() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(10.0, 40.0, 15.0, 5.0, 5.0, 2.0, 0.5, 0.5, 0.3, 0.02, 0.01, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.slag_type.contains("碱性") || result.slag_type.contains("Basic"));
    }

    #[test]
    fn test_slag_type_classification_neutral() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 30.0, 5.0, 8.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.slag_type.contains("中性") || result.slag_type.contains("Neutral"));
    }

    #[test]
    fn test_process_inference_han_furnace() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: Some(FurnaceType::HanChaogang),
        };
        let result = system.analyze(&request);
        assert!(result.process_inference.smelting_period.contains("汉代"));
        assert!(result.process_inference.estimated_temp_c > 0.0);
        assert!(!result.process_inference.reduction_atmosphere.is_empty());
        assert!(!result.process_inference.process_type.is_empty());
    }

    #[test]
    fn test_process_inference_ming_furnace() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: Some(FurnaceType::MingBlast),
        };
        let result = system.analyze(&request);
        assert!(result.process_inference.smelting_period.contains("明代"));
    }

    #[test]
    fn test_process_inference_high_sulfur_coal_hint() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 2.0, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.process_inference.fuel_type_hint.contains("煤"));
    }

    #[test]
    fn test_process_inference_low_sulfur_charcoal_hint() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.001, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.process_inference.fuel_type_hint.contains("木炭"));
    }

    #[test]
    fn test_process_inference_high_feo_low_reduction() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(25.0, 10.0, 5.0, 8.0, 30.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.process_inference.reduction_level < 0.5);
    }

    #[test]
    fn test_process_inference_low_feo_high_reduction() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(40.0, 25.0, 8.0, 10.0, 2.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.process_inference.reduction_level > 0.7);
    }

    #[test]
    fn test_process_inference_confidence_bounds() {
        let system = SlagAnalysisSystem::new();
        for _ in 0..5 {
            let request = SlagAnalysisRequest {
                composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
                furnace_type: None,
            };
            let result = system.analyze(&request);
            assert!(result.process_inference.confidence >= 0.2 && result.process_inference.confidence <= 0.95);
        }
    }

    #[test]
    fn test_ore_source_matching_normal() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(!result.ore_source_candidates.is_empty());
        for candidate in &result.ore_source_candidates {
            assert!(candidate.match_score > 0.0 && candidate.match_score <= 1.0);
            assert!(!candidate.region.is_empty());
        }
    }

    #[test]
    fn test_ore_source_matching_sorted_by_score() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        for window in result.ore_source_candidates.windows(2) {
            assert!(window[0].match_score >= window[1].match_score);
        }
    }

    #[test]
    fn test_ore_source_matching_max_five() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.ore_source_candidates.len() <= 5);
    }

    #[test]
    fn test_ore_source_matching_panzhihua_high_ti() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(30.0, 10.0, 5.0, 8.0, 8.0, 2.0, 0.5, 0.3, 5.0, 1.0, 0.5, 0.1),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        let best = result.ore_source_candidates.first();
        if let Some(candidate) = best {
            assert!(candidate.region.contains("攀枝花") || candidate.region.contains("哈密"));
        }
    }

    #[test]
    fn test_ore_source_matching_all_ore_sources_count() {
        let system = SlagAnalysisSystem::new();
        let sources = system.all_ore_sources();
        assert_eq!(sources.len(), 8);
        for s in &sources {
            assert!(!s.region.is_empty());
            assert!(!s.ore_type.is_empty());
        }
    }

    #[test]
    fn test_iron_quality_estimate_normal() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 5.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.iron_quality_estimate > 0.2 && result.iron_quality_estimate <= 0.98);
    }

    #[test]
    fn test_iron_quality_high_feo_lower() {
        let system = SlagAnalysisSystem::new();
        let req_low_feo = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 3.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let req_high_feo = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 20.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let res_low = system.analyze(&req_low_feo);
        let res_high = system.analyze(&req_high_feo);
        assert!(res_low.iron_quality_estimate > res_high.iron_quality_estimate);
    }

    #[test]
    fn test_generate_slag_sample_normal() {
        let system = SlagAnalysisSystem::new();
        let sample = system.generate_slag_sample(
            "河北邯郸",
            crate::models::FuelType::Charcoal,
            1300.0,
            0.7,
        );
        let total = sample.total();
        assert!((total - 1.0).abs() < 0.1, "sample total should be ~1.0, got {}", total);
        assert!(sample.sio2 > 0.0);
        assert!(sample.tio2 > 0.0);
    }

    #[test]
    fn test_generate_slag_sample_unknown_ore_source() {
        let system = SlagAnalysisSystem::new();
        let sample = system.generate_slag_sample(
            "不存在矿源",
            crate::models::FuelType::Charcoal,
            1300.0,
            0.7,
        );
        let total = sample.total();
        assert!((total - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_generate_slag_sample_coal_vs_charcoal_sulfur() {
        let system = SlagAnalysisSystem::new();
        let sample_charcoal = system.generate_slag_sample("河北邯郸", crate::models::FuelType::Charcoal, 1300.0, 0.7);
        let sample_coal = system.generate_slag_sample("河北邯郸", crate::models::FuelType::Coal, 1300.0, 0.7);
        assert!(sample_coal.s_content > sample_charcoal.s_content);
    }

    #[test]
    fn test_generate_slag_sample_higher_reduction_lower_feo() {
        let system = SlagAnalysisSystem::new();
        let sample_low = system.generate_slag_sample("河北邯郸", crate::models::FuelType::Charcoal, 1300.0, 0.3);
        let sample_high = system.generate_slag_sample("河北邯郸", crate::models::FuelType::Charcoal, 1300.0, 0.9);
        assert!(sample_high.feo < sample_low.feo);
    }

    #[test]
    fn test_generate_slag_sample_roundtrip() {
        let system = SlagAnalysisSystem::new();
        let sample = system.generate_slag_sample("四川攀枝花", crate::models::FuelType::Charcoal, 1300.0, 0.6);
        let request = SlagAnalysisRequest {
            composition: sample.clone(),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(!result.ore_source_candidates.is_empty());
        assert!(result.ore_source_candidates[0].region.contains("攀枝花") || result.ore_source_candidates[0].match_score > 0.4);
    }

    #[test]
    fn test_analysis_boundary_all_sio2() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.basicity < 0.01);
        assert!(result.melting_point_c >= 900.0);
    }

    #[test]
    fn test_analysis_boundary_all_cao() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(0.01, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.basicity > 10.0);
    }

    #[test]
    fn test_analysis_boundary_all_zero() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(result.melting_point_c >= 900.0);
    }

    #[test]
    fn test_analysis_evidence_not_empty() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.8, 0.5, 0.03, 0.02, 0.01),
            furnace_type: Some(FurnaceType::HanChaogang),
        };
        let result = system.analyze(&request);
        assert!(!result.process_inference.evidence.is_empty());
    }

    #[test]
    fn test_analysis_summary_not_empty() {
        let system = SlagAnalysisSystem::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01),
            furnace_type: None,
        };
        let result = system.analyze(&request);
        assert!(!result.analysis_summary.is_empty());
        assert!(result.analysis_summary.contains("碱度"));
    }
}
