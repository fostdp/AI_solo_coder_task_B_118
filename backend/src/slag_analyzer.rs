use crate::models::{
    FurnaceType, IronQualityEstimate, OreSourceMatch, ProcessInference, SlagAnalysisRequest,
    SlagAnalysisResult, SlagComposition, SlagType,
};
use crate::slag::{SlagAnalysisSystem, bayesian_posterior, gaussian_likelihood};
use crate::models::FuelType;

pub struct SlagAnalyzer {
    system: SlagAnalysisSystem,
}

impl SlagAnalyzer {
    pub fn new() -> Self {
        Self {
            system: SlagAnalysisSystem::new(),
        }
    }

    pub fn analyze(&self, request: &SlagAnalysisRequest) -> SlagAnalysisResult {
        self.system.analyze(request)
    }

    pub fn classify_slag(&self, composition: &SlagComposition) -> SlagType {
        let normalized = composition.normalize();
        let basicity = normalized.basicity();
        let quaternary_basicity = normalized.quaternary_basicity();
        self.system.classify_slag_type(basicity, quaternary_basicity)
    }

    pub fn infer_process_bayesian(
        &self,
        composition: &SlagComposition,
        furnace_type: Option<FurnaceType>,
    ) -> ProcessInference {
        let normalized = composition.normalize();
        let basicity = normalized.basicity();
        self.system
            .infer_process(&normalized, basicity, furnace_type)
    }

    pub fn match_ore_sources(&self, composition: &SlagComposition) -> Vec<OreSourceMatch> {
        let normalized = composition.normalize();
        self.system.match_ore_sources(&normalized)
    }

    pub fn estimate_iron_quality(&self, composition: &SlagComposition) -> IronQualityEstimate {
        let normalized = composition.normalize();
        self.system.estimate_iron_quality(&normalized)
    }

    pub fn generate_sample(
        &self,
        ore_source_name: &str,
        furnace_type: Option<FurnaceType>,
        reduction_level: f64,
    ) -> SlagComposition {
        let (fuel_type, temp_c) = match furnace_type {
            Some(FurnaceType::HanChaogang) => (FuelType::Charcoal, 1200.0),
            Some(FurnaceType::MingBlast) => (FuelType::Coal, 1350.0),
            None => (FuelType::Charcoal, 1250.0),
        };
        self.system
            .generate_slag_sample(ore_source_name, fuel_type, temp_c, reduction_level)
    }

    pub fn calculate_basicity(&self, composition: &SlagComposition) -> f64 {
        let normalized = composition.normalize();
        normalized.basicity()
    }

    pub fn calculate_quaternary_basicity(&self, composition: &SlagComposition) -> f64 {
        let normalized = composition.normalize();
        normalized.quaternary_basicity()
    }

    pub fn estimate_melting_point(&self, composition: &SlagComposition) -> f64 {
        let normalized = composition.normalize();
        let basicity = normalized.basicity();
        self.system.estimate_melting_point(&normalized, basicity)
    }

    pub fn bayesian_posterior(
        &self,
        prior: f64,
        likelihood: f64,
        evidence_norm: f64,
    ) -> f64 {
        bayesian_posterior(prior, likelihood, evidence_norm)
    }

    pub fn gaussian_likelihood(&self, x: f64, mean: f64, std: f64) -> f64 {
        gaussian_likelihood(x, mean, std)
    }

    pub fn all_ore_sources(&self) -> Vec<OreSourceMatch> {
        self.system.all_ore_sources()
    }

    pub fn generate_sample_with_params(
        &self,
        ore_source_name: &str,
        fuel_type: FuelType,
        temp_c: f64,
        reduction_level: f64,
    ) -> SlagComposition {
        self.system
            .generate_slag_sample(ore_source_name, fuel_type, temp_c, reduction_level)
    }
}

impl Default for SlagAnalyzer {
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

    fn make_composition(
        sio2: f64,
        cao: f64,
        mgo: f64,
        al2o3: f64,
        feo: f64,
        mno: f64,
        p2o5: f64,
        s: f64,
        tio2: f64,
        v2o5: f64,
        cr2o3: f64,
        ni_o: f64,
    ) -> SlagComposition {
        SlagComposition {
            sio2,
            cao,
            mgo,
            al2o3,
            feo,
            mno,
            p2o5,
            s_content: s,
            tio2,
            v2o5,
            cr2o3,
            ni_o,
        }
    }

    #[test]
    fn test_basicity_neutral() {
        let comp = make_composition(
            0.35, 0.35, 0.05, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let normalized = comp.normalize();
        let b = normalized.basicity();
        assert!(
            approx_eq(b, 1.0, 0.05),
            "basicity should be ~1.0, got {}",
            b
        );
    }

    #[test]
    fn test_basicity_acidic() {
        let comp = make_composition(
            0.50, 0.10, 0.03, 0.10, 0.15, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let normalized = comp.normalize();
        let b = normalized.basicity();
        assert!(b < 1.0, "acidic slag should have basicity < 1.0, got {}", b);
    }

    #[test]
    fn test_basicity_basic() {
        let comp = make_composition(
            0.15, 0.40, 0.10, 0.05, 0.10, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let normalized = comp.normalize();
        let b = normalized.basicity();
        assert!(b > 1.0, "basic slag should have basicity > 1.0, got {}", b);
    }

    #[test]
    fn test_basicity_zero_sio2() {
        let comp = make_composition(
            0.0, 0.35, 0.05, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let b = comp.basicity();
        assert_eq!(b, 0.0, "basicity with zero SiO2 should be 0.0");
    }

    #[test]
    fn test_quaternary_basicity() {
        let comp = make_composition(
            0.30, 0.25, 0.10, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let normalized = comp.normalize();
        let qb = normalized.quaternary_basicity();
        assert!(qb > 0.0);
        let expected =
            (normalized.cao + normalized.mgo) / (normalized.sio2 + normalized.al2o3);
        assert!(approx_eq(qb, expected, 1e-6));
    }

    #[test]
    fn test_quaternary_basicity_zero_denominator() {
        let comp = SlagComposition {
            sio2: 0.0,
            al2o3: 0.0,
            ..SlagComposition::default()
        };
        let qb = comp.quaternary_basicity();
        assert_eq!(qb, 0.0);
    }

    #[test]
    fn test_normalize_sum_to_one() {
        let comp = make_composition(
            35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01,
        );
        let normalized = comp.normalize();
        let total = normalized.total();
        assert!(
            approx_eq(total, 1.0, 0.001),
            "normalized total should be 1.0, got {}",
            total
        );
    }

    #[test]
    fn test_normalize_zero_total() {
        let comp = make_composition(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        let normalized = comp.normalize();
        assert!(approx_eq(normalized.total(), 0.0, 1e-6));
    }

    #[test]
    fn test_full_analysis_normal() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01,
            ),
            furnace_type: Some(FurnaceType::HanChaogang),
        };
        let result = analyzer.analyze(&request);

        assert!(result.basicity > 0.0);
        assert!(!result.slag_type.is_empty());
        assert!(result.melting_point_c >= 900.0 && result.melting_point_c <= 1600.0);
        assert!(result.viscosity_pa_s >= 0.01 && result.viscosity_pa_s <= 50.0);
        assert!(
            result.process_inference.confidence > 0.0
                && result.process_inference.confidence <= 0.95
        );
        assert!(!result.analysis_summary.is_empty());
    }

    #[test]
    fn test_analysis_melting_point_range() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.melting_point_c >= 900.0 && result.melting_point_c <= 1600.0);
    }

    #[test]
    fn test_analysis_viscosity_above_melting_point() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                30.0, 20.0, 10.0, 8.0, 5.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.viscosity_pa_s > 0.0);
        assert!(result.viscosity_pa_s < 100.0);
    }

    #[test]
    fn test_slag_type_classification_acidic() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                50.0, 5.0, 2.0, 15.0, 10.0, 2.0, 0.5, 0.5, 1.0, 0.05, 0.03, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(
            result.slag_type.contains("酸性") || result.slag_type.contains("Acidic")
        );
    }

    #[test]
    fn test_slag_type_classification_basic() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                10.0, 40.0, 15.0, 5.0, 5.0, 2.0, 0.5, 0.5, 0.3, 0.02, 0.01, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(
            result.slag_type.contains("碱性") || result.slag_type.contains("Basic")
        );
    }

    #[test]
    fn test_slag_type_classification_neutral() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 30.0, 5.0, 8.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(
            result.slag_type.contains("中性") || result.slag_type.contains("Neutral")
        );
    }

    #[test]
    fn test_process_inference_han_furnace() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: Some(FurnaceType::HanChaogang),
        };
        let result = analyzer.analyze(&request);
        assert!(result.process_inference.smelting_period.contains("汉代"));
        assert!(result.process_inference.estimated_temp_c > 0.0);
        assert!(!result.process_inference.reduction_atmosphere.is_empty());
        assert!(!result.process_inference.process_type.is_empty());
    }

    #[test]
    fn test_process_inference_ming_furnace() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: Some(FurnaceType::MingBlast),
        };
        let result = analyzer.analyze(&request);
        assert!(result.process_inference.smelting_period.contains("明代"));
    }

    #[test]
    fn test_process_inference_high_sulfur_coal_hint() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 2.0, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.process_inference.fuel_type_hint.contains("煤"));
    }

    #[test]
    fn test_process_inference_low_sulfur_charcoal_hint() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.001, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.process_inference.fuel_type_hint.contains("木炭"));
    }

    #[test]
    fn test_process_inference_high_feo_low_reduction() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                25.0, 10.0, 5.0, 8.0, 30.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.process_inference.reduction_level < 0.5);
    }

    #[test]
    fn test_process_inference_low_feo_high_reduction() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                40.0, 25.0, 8.0, 10.0, 2.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.process_inference.reduction_level > 0.7);
    }

    #[test]
    fn test_process_inference_confidence_bounds() {
        let analyzer = SlagAnalyzer::new();
        for _ in 0..5 {
            let request = SlagAnalysisRequest {
                composition: make_composition(
                    35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
                ),
                furnace_type: None,
            };
            let result = analyzer.analyze(&request);
            assert!(
                result.process_inference.confidence >= 0.2
                    && result.process_inference.confidence <= 0.95
            );
        }
    }

    #[test]
    fn test_ore_source_matching_normal() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(!result.ore_source_candidates.is_empty());
        for candidate in &result.ore_source_candidates {
            assert!(candidate.match_score > 0.0 && candidate.match_score <= 1.0);
            assert!(!candidate.region.is_empty());
        }
    }

    #[test]
    fn test_ore_source_matching_sorted_by_score() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        for window in result.ore_source_candidates.windows(2) {
            assert!(window[0].match_score >= window[1].match_score);
        }
    }

    #[test]
    fn test_ore_source_matching_max_five() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.ore_source_candidates.len() <= 5);
    }

    #[test]
    fn test_ore_source_matching_panzhihua_high_ti() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                30.0, 10.0, 5.0, 8.0, 8.0, 2.0, 0.5, 0.3, 5.0, 1.0, 0.5, 0.1,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        let best = result.ore_source_candidates.first();
        if let Some(candidate) = best {
            assert!(
                candidate.region.contains("攀枝花") || candidate.region.contains("哈密")
            );
        }
    }

    #[test]
    fn test_ore_source_matching_all_ore_sources_count() {
        let analyzer = SlagAnalyzer::new();
        let sources = analyzer.all_ore_sources();
        assert_eq!(sources.len(), 8);
        for s in &sources {
            assert!(!s.region.is_empty());
            assert!(!s.ore_type.is_empty());
        }
    }

    #[test]
    fn test_iron_quality_estimate_normal() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 5.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(
            result.iron_quality_estimate > 0.2 && result.iron_quality_estimate <= 0.98
        );
    }

    #[test]
    fn test_iron_quality_high_feo_lower() {
        let analyzer = SlagAnalyzer::new();
        let req_low_feo = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 3.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let req_high_feo = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 20.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let res_low = analyzer.analyze(&req_low_feo);
        let res_high = analyzer.analyze(&req_high_feo);
        assert!(res_low.iron_quality_estimate > res_high.iron_quality_estimate);
    }

    #[test]
    fn test_generate_slag_sample_normal() {
        let analyzer = SlagAnalyzer::new();
        let sample = analyzer.generate_sample("河北邯郸", Some(FurnaceType::HanChaogang), 0.7);
        let total = sample.total();
        assert!(
            (total - 1.0).abs() < 0.1,
            "sample total should be ~1.0, got {}",
            total
        );
        assert!(sample.sio2 > 0.0);
        assert!(sample.tio2 > 0.0);
    }

    #[test]
    fn test_generate_slag_sample_unknown_ore_source() {
        let analyzer = SlagAnalyzer::new();
        let sample = analyzer.generate_sample("不存在矿源", None, 0.7);
        let total = sample.total();
        assert!((total - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_generate_slag_sample_coal_vs_charcoal_sulfur() {
        let analyzer = SlagAnalyzer::new();
        let sample_charcoal =
            analyzer.generate_sample("河北邯郸", Some(FurnaceType::HanChaogang), 0.7);
        let sample_coal = analyzer.generate_sample("河北邯郸", Some(FurnaceType::MingBlast), 0.7);
        assert!(sample_coal.s_content > sample_charcoal.s_content);
    }

    #[test]
    fn test_generate_slag_sample_higher_reduction_lower_feo() {
        let analyzer = SlagAnalyzer::new();
        let sample_low = analyzer.generate_sample("河北邯郸", None, 0.3);
        let sample_high = analyzer.generate_sample("河北邯郸", None, 0.9);
        assert!(sample_high.feo < sample_low.feo);
    }

    #[test]
    fn test_generate_slag_sample_roundtrip() {
        let analyzer = SlagAnalyzer::new();
        let sample = analyzer.generate_sample("四川攀枝花", Some(FurnaceType::HanChaogang), 0.6);
        let request = SlagAnalysisRequest {
            composition: sample.clone(),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(!result.ore_source_candidates.is_empty());
        assert!(
            result.ore_source_candidates[0].region.contains("攀枝花")
                || result.ore_source_candidates[0].match_score > 0.4
        );
    }

    #[test]
    fn test_analysis_boundary_all_sio2() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.basicity < 0.01);
        assert!(result.melting_point_c >= 900.0);
    }

    #[test]
    fn test_analysis_boundary_all_cao() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                0.01, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.basicity > 10.0);
    }

    #[test]
    fn test_analysis_boundary_all_zero() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(result.melting_point_c >= 900.0);
    }

    #[test]
    fn test_analysis_evidence_not_empty() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.8, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: Some(FurnaceType::HanChaogang),
        };
        let result = analyzer.analyze(&request);
        assert!(!result.process_inference.evidence.is_empty());
    }

    #[test]
    fn test_analysis_summary_not_empty() {
        let analyzer = SlagAnalyzer::new();
        let request = SlagAnalysisRequest {
            composition: make_composition(
                35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
            ),
            furnace_type: None,
        };
        let result = analyzer.analyze(&request);
        assert!(!result.analysis_summary.is_empty());
        assert!(result.analysis_summary.contains("碱度"));
    }

    #[test]
    fn test_classify_slag_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            0.35, 0.35, 0.05, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let slag_type = analyzer.classify_slag(&comp);
        assert!(!slag_type.is_empty());
    }

    #[test]
    fn test_infer_process_bayesian_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
        );
        let inference = analyzer.infer_process_bayesian(&comp, Some(FurnaceType::HanChaogang));
        assert!(!inference.process_type.is_empty());
        assert!(inference.confidence > 0.0);
    }

    #[test]
    fn test_match_ore_sources_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.25, 0.10, 0.02, 0.01,
        );
        let matches = analyzer.match_ore_sources(&comp);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_estimate_iron_quality_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            35.0, 15.0, 8.0, 10.0, 5.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
        );
        let quality = analyzer.estimate_iron_quality(&comp);
        assert!(quality > 0.2 && quality <= 0.98);
    }

    #[test]
    fn test_calculate_basicity_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            0.35, 0.35, 0.05, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let b = analyzer.calculate_basicity(&comp);
        assert!(b > 0.0);
    }

    #[test]
    fn test_calculate_quaternary_basicity_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            0.30, 0.25, 0.10, 0.10, 0.08, 0.02, 0.01, 0.005, 0.01, 0.003, 0.002, 0.001,
        );
        let qb = analyzer.calculate_quaternary_basicity(&comp);
        assert!(qb > 0.0);
    }

    #[test]
    fn test_estimate_melting_point_helper() {
        let analyzer = SlagAnalyzer::new();
        let comp = make_composition(
            35.0, 15.0, 8.0, 10.0, 12.0, 2.0, 0.5, 0.8, 0.6, 0.05, 0.03, 0.01,
        );
        let mp = analyzer.estimate_melting_point(&comp);
        assert!(mp >= 900.0 && mp <= 1600.0);
    }

    #[test]
    fn test_bayesian_helpers_available() {
        let analyzer = SlagAnalyzer::new();
        let post = analyzer.bayesian_posterior(0.5, 0.8, 0.6);
        assert!(post > 0.0 && post <= 1.0);
        let like = analyzer.gaussian_likelihood(0.5, 0.5, 0.1);
        assert!(like > 0.0 && like <= 1.0);
    }

    #[test]
    fn test_default_impl() {
        let analyzer = SlagAnalyzer::default();
        let comp = make_composition(
            35.0, 15.0, 8.0, 10.0, 8.0, 2.0, 0.5, 0.3, 0.5, 0.03, 0.02, 0.01,
        );
        let b = analyzer.calculate_basicity(&comp);
        assert!(b > 0.0);
    }
}
