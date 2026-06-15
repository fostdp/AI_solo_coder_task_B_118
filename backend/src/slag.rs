use tracing::debug;

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

pub fn bayesian_posterior(prior: f64, likelihood: f64, evidence_norm: f64) -> f64 {
    if evidence_norm <= 0.0 { prior } else { (prior * likelihood / evidence_norm).min(0.999).max(0.001) }
}

pub fn gaussian_likelihood(x: f64, mean: f64, std: f64) -> f64 {
    let z = (x - mean) / std;
    (-0.5 * z * z).exp()
}

#[derive(Debug, Clone)]
pub struct BayesianHypothesis {
    pub name: &'static str,
    pub prior: f64,
    pub feo_mean: f64,
    pub feo_std: f64,
    pub s_mean: f64,
    pub s_std: f64,
    pub basicity_mean: f64,
    pub basicity_std: f64,
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

    pub fn estimate_melting_point(&self, composition: &SlagComposition, basicity: f64) -> f64 {
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

    pub fn classify_slag_type(&self, basicity: f64, _quaternary_basicity: f64) -> String {
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

    pub fn infer_process(
        &self,
        composition: &SlagComposition,
        basicity: f64,
        furnace_type: Option<crate::models::FurnaceType>,
    ) -> ProcessInference {
        let mut evidence = Vec::new();

        let estimated_temp = 1200.0 + (basicity - 1.0) * 200.0 + composition.feo * -500.0;
        let estimated_temp = estimated_temp.max(900.0).min(1600.0);
        let temp_conf = 0.6 + (1.0 - (composition.feo - 0.05).abs()) * 0.3;

        let reduction_hypotheses = vec![
            BayesianHypothesis {
                name: "高还原度",
                prior: 0.4,
                feo_mean: 0.04,
                feo_std: 0.02,
                s_mean: 0.0,
                s_std: 1.0,
                basicity_mean: 0.0,
                basicity_std: 1.0,
            },
            BayesianHypothesis {
                name: "中还原度",
                prior: 0.35,
                feo_mean: 0.10,
                feo_std: 0.03,
                s_mean: 0.0,
                s_std: 1.0,
                basicity_mean: 0.0,
                basicity_std: 1.0,
            },
            BayesianHypothesis {
                name: "低还原度",
                prior: 0.25,
                feo_mean: 0.18,
                feo_std: 0.04,
                s_mean: 0.0,
                s_std: 1.0,
                basicity_mean: 0.0,
                basicity_std: 1.0,
            },
        ];

        let reduction_likelihoods: Vec<f64> = reduction_hypotheses
            .iter()
            .map(|h| gaussian_likelihood(composition.feo, h.feo_mean, h.feo_std))
            .collect();
        let reduction_evidence: f64 = reduction_hypotheses
            .iter()
            .zip(reduction_likelihoods.iter())
            .map(|(h, l)| h.prior * l)
            .sum();
        let reduction_posteriors: Vec<f64> = reduction_hypotheses
            .iter()
            .zip(reduction_likelihoods.iter())
            .map(|(h, l)| bayesian_posterior(h.prior, *l, reduction_evidence))
            .collect();
        let (reduction_idx, &reduction_confidence) = reduction_posteriors
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        let reduction_name = reduction_hypotheses[reduction_idx].name;
        let reduction_level = match reduction_idx {
            0 => 0.85,
            1 => 0.6,
            _ => 0.3,
        };
        let reduction_desc = match reduction_idx {
            0 => "还原充分",
            1 => "还原程度一般",
            _ => "还原程度较低",
        };
        evidence.push(format!(
            "FeO={:.3}, P({})={:.2}%, {}",
            composition.feo,
            reduction_name,
            reduction_confidence * 100.0,
            reduction_desc
        ));

        let fuel_hypotheses = vec![
            BayesianHypothesis {
                name: "木炭",
                prior: 0.5,
                feo_mean: 0.0,
                feo_std: 1.0,
                s_mean: 0.001,
                s_std: 0.001,
                basicity_mean: 0.0,
                basicity_std: 1.0,
            },
            BayesianHypothesis {
                name: "木炭为主，可能混用少量煤",
                prior: 0.3,
                feo_mean: 0.0,
                feo_std: 1.0,
                s_mean: 0.006,
                s_std: 0.003,
                basicity_mean: 0.0,
                basicity_std: 1.0,
            },
            BayesianHypothesis {
                name: "煤炭/焦炭",
                prior: 0.2,
                feo_mean: 0.0,
                feo_std: 1.0,
                s_mean: 0.018,
                s_std: 0.006,
                basicity_mean: 0.0,
                basicity_std: 1.0,
            },
        ];

        let fuel_likelihoods: Vec<f64> = fuel_hypotheses
            .iter()
            .map(|h| gaussian_likelihood(composition.s_content, h.s_mean, h.s_std))
            .collect();
        let fuel_evidence: f64 = fuel_hypotheses
            .iter()
            .zip(fuel_likelihoods.iter())
            .map(|(h, l)| h.prior * l)
            .sum();
        let fuel_posteriors: Vec<f64> = fuel_hypotheses
            .iter()
            .zip(fuel_likelihoods.iter())
            .map(|(h, l)| bayesian_posterior(h.prior, *l, fuel_evidence))
            .collect();
        let (fuel_idx, &fuel_confidence) = fuel_posteriors
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        let fuel_hint = fuel_hypotheses[fuel_idx].name.to_string();
        let fuel_desc = match fuel_idx {
            0 => "低硫特征，推测使用木炭",
            1 => "硫含量中等，可能是木炭或混合燃料",
            _ => "高硫特征，推测使用煤炭或焦炭",
        };
        evidence.push(format!(
            "S={:.3}, P({})={:.2}%, {}",
            composition.s_content,
            fuel_hypotheses[fuel_idx].name,
            fuel_confidence * 100.0,
            fuel_desc
        ));

        let process_hypotheses = vec![
            BayesianHypothesis {
                name: "酸性熔炼法",
                prior: 0.3,
                feo_mean: 0.0,
                feo_std: 1.0,
                s_mean: 0.0,
                s_std: 1.0,
                basicity_mean: 0.4,
                basicity_std: 0.15,
            },
            BayesianHypothesis {
                name: "中性熔炼法",
                prior: 0.4,
                feo_mean: 0.0,
                feo_std: 1.0,
                s_mean: 0.0,
                s_std: 1.0,
                basicity_mean: 1.0,
                basicity_std: 0.2,
            },
            BayesianHypothesis {
                name: "碱性熔炼法",
                prior: 0.3,
                feo_mean: 0.0,
                feo_std: 1.0,
                s_mean: 0.0,
                s_std: 1.0,
                basicity_mean: 1.8,
                basicity_std: 0.3,
            },
        ];

        let process_likelihoods: Vec<f64> = process_hypotheses
            .iter()
            .map(|h| gaussian_likelihood(basicity, h.basicity_mean, h.basicity_std))
            .collect();
        let process_evidence: f64 = process_hypotheses
            .iter()
            .zip(process_likelihoods.iter())
            .map(|(h, l)| h.prior * l)
            .sum();
        let process_posteriors: Vec<f64> = process_hypotheses
            .iter()
            .zip(process_likelihoods.iter())
            .map(|(h, l)| bayesian_posterior(h.prior, *l, process_evidence))
            .collect();
        let (process_idx, &process_confidence) = process_posteriors
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        let process_type = process_hypotheses[process_idx].name.to_string();
        let process_desc = match process_idx {
            0 => "酸性渣，可能是未加熔剂的自然熔炼",
            1 => "中性熔炼",
            _ => "高碱度渣，可能使用了石灰石作熔剂",
        };
        evidence.push(format!(
            "碱度={:.3}, P({})={:.2}%, {}",
            basicity,
            process_type,
            process_confidence * 100.0,
            process_desc
        ));

        let reduction_atmosphere = match fuel_idx {
            0 => "强还原性".to_string(),
            1 => "中等还原性".to_string(),
            _ => "弱还原性".to_string(),
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

        let confidence = ((reduction_confidence + fuel_confidence + process_confidence) / 3.0)
            .min(0.95)
            .max(0.2);

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

    pub fn match_ore_sources(&self, composition: &SlagComposition) -> Vec<OreSourceCandidate> {
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

    pub fn estimate_iron_quality(&self, composition: &SlagComposition) -> f64 {
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
