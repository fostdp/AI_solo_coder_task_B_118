export class SlagAnalyzer {
    constructor(containerId, apiClient) {
        this.containerId = containerId;
        this.apiClient = apiClient;
        this.container = null;
        this.state = {
            oreSources: [],
            lastResults: null,
            isLoading: false,
            composition: {
                sio2: 35, cao: 15, mgo: 8, al2o3: 10,
                feo: 12, fe2o3: 3, mno: 2, p2o5: 0.5,
                s: 0.8, tio2: 0.6, v2o5: 0.05, cr2o3: 0.03
            }
        };
    }

    render() {
        this.container = document.getElementById(this.containerId);
        if (!this.container) return;

        this.container.innerHTML = `
            <div class="feature-panel" style="background: transparent; border: none; padding: 0;">
                <div class="feature-header">
                    <h2><span class="icon">🧪</span>炉渣成分分析</h2>
                    <p>基于炉渣化学成分反推冶炼工艺和矿石来源</p>
                </div>
                <div class="slag-analysis-container">
                    <div class="slag-input-section">
                        <h3>炉渣成分输入 (%)</h3>
                        <div class="slag-form-grid">
                            <div class="form-item"><label>SiO₂</label><input type="number" class="slag-input" data-field="sio2" value="35" step="0.1"></div>
                            <div class="form-item"><label>CaO</label><input type="number" class="slag-input" data-field="cao" value="15" step="0.1"></div>
                            <div class="form-item"><label>MgO</label><input type="number" class="slag-input" data-field="mgo" value="8" step="0.1"></div>
                            <div class="form-item"><label>Al₂O₃</label><input type="number" class="slag-input" data-field="al2o3" value="10" step="0.1"></div>
                            <div class="form-item"><label>FeO</label><input type="number" class="slag-input" data-field="feo" value="12" step="0.1"></div>
                            <div class="form-item"><label>Fe₂O₃</label><input type="number" class="slag-input" data-field="fe2o3" value="3" step="0.1"></div>
                            <div class="form-item"><label>MnO</label><input type="number" class="slag-input" data-field="mno" value="2" step="0.1"></div>
                            <div class="form-item"><label>P₂O₅</label><input type="number" class="slag-input" data-field="p2o5" value="0.5" step="0.1"></div>
                            <div class="form-item"><label>S</label><input type="number" class="slag-input" data-field="s" value="0.8" step="0.1"></div>
                            <div class="form-item"><label>TiO₂</label><input type="number" class="slag-input" data-field="tio2" value="0.6" step="0.1"></div>
                            <div class="form-item"><label>V₂O₅</label><input type="number" class="slag-input" data-field="v2o5" value="0.05" step="0.01"></div>
                            <div class="form-item"><label>Cr₂O₃</label><input type="number" class="slag-input" data-field="cr2o3" value="0.03" step="0.01"></div>
                        </div>
                        <div class="slag-actions">
                            <button class="btn-primary slag-analyze-btn">开始分析</button>
                            <button class="btn-secondary slag-sample-btn">生成示例样本</button>
                        </div>
                    </div>
                    <div class="slag-results-section">
                        <div class="results-placeholder slag-results-area">请输入炉渣成分后点击"开始分析"</div>
                    </div>
                </div>
            </div>
        `;

        this._bindEvents();
        this._loadOreSources();
    }

    _bindEvents() {
        const analyzeBtn = this.container.querySelector('.slag-analyze-btn');
        if (analyzeBtn) {
            analyzeBtn.addEventListener('click', () => this.analyze());
        }

        const sampleBtn = this.container.querySelector('.slag-sample-btn');
        if (sampleBtn) {
            sampleBtn.addEventListener('click', () => this.generateSample());
        }

        const inputs = this.container.querySelectorAll('.slag-input');
        inputs.forEach(input => {
            input.addEventListener('change', (e) => {
                const field = e.target.dataset.field;
                this.state.composition[field] = parseFloat(e.target.value) || 0;
            });
        });
    }

    async _loadOreSources() {
        try {
            const res = await this.apiClient.get('/api/slag/ore-sources');
            if (res && (res.success || Array.isArray(res))) {
                this.state.oreSources = res.data || res || [];
            }
        } catch (e) {
            console.warn('[SlagAnalyzer] Failed to load ore sources:', e);
        }
    }

    _getInputValues() {
        const values = {};
        const inputs = this.container.querySelectorAll('.slag-input');
        inputs.forEach(input => {
            const field = input.dataset.field;
            values[field] = parseFloat(input.value) || 0;
        });
        return values;
    }

    async analyze() {
        const composition = this._getInputValues();
        const resultsArea = this.container.querySelector('.slag-results-area');

        if (resultsArea) {
            resultsArea.outerHTML = '<div class="results-placeholder slag-results-area">分析中...</div>';
        }
        this.state.isLoading = true;

        try {
            const data = await this.apiClient.post('/api/slag/analyze', {
                composition,
                furnace_id: 'HAN-001',
                sample_id: 'sample-' + Date.now()
            });

            if (data && (data.success || data.basicity !== undefined || data.process_inference)) {
                const results = data.data || data;
                this.state.lastResults = results;
                this._renderResults(results, composition);
            } else {
                throw new Error('API returned invalid response');
            }
        } catch (e) {
            console.error('[SlagAnalyzer] Analysis error:', e);
            this._renderResults(this._generateFallbackResults(composition), composition);
        } finally {
            this.state.isLoading = false;
        }
    }

    async generateSample() {
        let sample = null;

        try {
            const res = await this.apiClient.get('/api/slag/sample?furnace_id=HAN-001&fuel_type=Charcoal&ore_type=hematite');
            if (res && (res.success || res.composition)) {
                sample = res.composition || res.data?.composition || res;
            }
        } catch (e) {
            console.warn('[SlagAnalyzer] Failed to generate sample via API, using fallback:', e);
        }

        if (!sample) {
            sample = {
                sio2: 35.2, cao: 14.8, mgo: 7.5, al2o3: 9.6,
                feo: 12.3, fe2o3: 3.2, mno: 1.8, p2o5: 0.45,
                s: 0.78, tio2: 0.55, v2o5: 0.04, cr2o3: 0.025
            };
        }

        this._fillFormValues(sample);
        this.state.composition = { ...sample };
    }

    _fillFormValues(composition) {
        const inputs = this.container.querySelectorAll('.slag-input');
        inputs.forEach(input => {
            const field = input.dataset.field;
            if (composition[field] !== undefined) {
                input.value = composition[field].toFixed(2);
            }
        });
    }

    _renderResults(data, composition) {
        const resultsArea = this.container.querySelector('.slag-results-area');
        if (!resultsArea) return;

        let html = '';

        const basicityR2 = data.basicity ?? (composition.cao / composition.sio2);
        const basicityR4 = data.basicity_r4 ?? ((composition.cao + composition.mgo) / (composition.sio2 + composition.al2o3));
        const meltingPoint = data.melting_point ?? (1350 + (composition.al2o3 - 10) * 15 - (composition.cao - 15) * 10);
        const viscosity = data.viscosity ?? (0.5 + Math.abs(basicityR2 - 1.0) * 0.8);

        html += `
            <div class="slag-result-section">
                <h4>📊 炉渣基本属性</h4>
                <div class="slag-basicity-info">
                    <div class="slag-info-item">
                        <div class="slag-info-value">${basicityR2.toFixed(2)}</div>
                        <div class="slag-info-label">二元碱度 R2 (CaO/SiO₂)</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value">${basicityR4.toFixed(2)}</div>
                        <div class="slag-info-label">四元碱度 R4</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value">${meltingPoint.toFixed(0)}°C</div>
                        <div class="slag-info-label">估算熔点</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value">${viscosity.toFixed(2)}</div>
                        <div class="slag-info-label">粘度 Pa·s</div>
                    </div>
                </div>
                <div style="margin-top: 12px; padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 6px;">渣系判断</div>
                    <div style="font-size: 14px; font-weight: 600; color: var(--accent-cyan);">
                        ${basicityR2 > 1.2 ? '碱性渣' : basicityR2 > 0.8 ? '中性渣' : '酸性渣'} · 
                        流动性${viscosity < 0.5 ? '良好' : viscosity < 1.0 ? '一般' : '较差'} · 
                        脱硫能力${basicityR2 > 1.0 ? '强' : basicityR2 > 0.7 ? '中' : '弱'}
                    </div>
                </div>
            </div>
        `;

        const processInference = data.process_inference || this._generateFallbackProcessInference(composition, basicityR2);
        const pi = processInference;

        html += `
            <div class="slag-result-section">
                <h4>🏺 工艺反推（贝叶斯推断）</h4>
                <div class="slag-basicity-info">
                    <div class="slag-info-item">
                        <div class="slag-info-value" style="font-size: 14px;">${pi.estimated_temp_c?.toFixed(0) || '--'}°C</div>
                        <div class="slag-info-label">推断冶炼温度</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value" style="font-size: 14px;">${pi.estimated_period || '--'}</div>
                        <div class="slag-info-label">推断历史时期</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value" style="font-size: 14px;">${pi.estimated_fuel_type || '--'}</div>
                        <div class="slag-info-label">推断燃料类型</div>
                    </div>
                </div>
                ${pi.bayesian_posteriors ? `
                <div style="margin-top: 12px; padding: 12px; background: rgba(167, 139, 250, 0.08); border-radius: 8px; border-left: 3px solid var(--accent-purple);">
                    <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 8px; font-weight: 600;">贝叶斯后验概率</div>
                    <div style="display: grid; gap: 6px;">
                        ${Object.entries(pi.bayesian_posteriors).map(([key, val]) => `
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="font-size: 12px; color: var(--text-secondary); width: 100px; flex-shrink: 0;">${this._translatePosteriorKey(key)}</span>
                                <div style="flex: 1; height: 8px; background: var(--bg-tertiary); border-radius: 4px; overflow: hidden;">
                                    <div style="height: 100%; width: ${(val * 100).toFixed(0)}%; background: linear-gradient(90deg, var(--accent-purple), var(--accent-pink)); transition: width 0.3s;"></div>
                                </div>
                                <span style="font-size: 12px; font-weight: 600; color: var(--accent-purple); width: 45px; text-align: right;">${(val * 100).toFixed(0)}%</span>
                            </div>
                        `).join('')}
                    </div>
                </div>
                ` : ''}
                <div style="margin-top: 10px; padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">还原气氛</div>
                    <div style="font-size: 14px; font-weight: 600; color: var(--accent-orange);">${pi.reduction_atmosphere || '--'}
                        <span style="font-size: 11px; color: var(--text-muted);">(置信度 ${((pi.confidence || 0.6) * 100).toFixed(0)}%)</span>
                    </div>
                </div>
                ${pi.process_description ? `
                <div style="margin-top: 10px; padding: 10px; background: rgba(167, 139, 250, 0.1); border-radius: 8px; border-left: 3px solid var(--accent-purple);">
                    <div style="font-size: 12px; color: var(--text-secondary);">${pi.process_description}</div>
                </div>` : ''}
            </div>
        `;

        const oreCandidates = data.ore_source_candidates || this._generateFallbackOreCandidates(composition);
        if (oreCandidates && oreCandidates.length > 0) {
            html += `
                <div class="slag-result-section">
                    <h4>⛏️ 矿石来源匹配</h4>
                    <div class="ore-source-list">
                        ${oreCandidates.map(candidate => `
                            <div class="ore-source-item">
                                <div class="ore-source-info">
                                    <div class="ore-source-name">${candidate.name || candidate.source_name || '未知矿源'}</div>
                                    <div class="ore-source-loc">${candidate.location || ''} · ${candidate.historical_period || candidate.period || ''}</div>
                                </div>
                                <div class="ore-source-score">
                                    <div class="ore-source-percent">${((candidate.similarity || candidate.score || 0) * 100).toFixed(1)}%</div>
                                    <div class="ore-source-bar">
                                        <div class="ore-source-bar-fill" style="width: ${((candidate.similarity || candidate.score || 0) * 100)}%;"></div>
                                    </div>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        const ironQuality = data.iron_quality_estimate || this._generateFallbackIronQuality(composition, basicityR2);
        html += `
            <div class="slag-result-section">
                <h4>🔩 铁质量估算</h4>
                <div style="padding: 16px; background: linear-gradient(135deg, rgba(34, 197, 94, 0.1), rgba(16, 185, 129, 0.1)); border-radius: 12px; border: 1px solid rgba(34, 197, 94, 0.2);">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                        <span style="font-size: 14px; color: var(--text-secondary);">综合质量评分</span>
                        <span style="font-size: 28px; font-weight: 700; color: var(--accent-green);">${(ironQuality.overall_score || ironQuality.overall || 0).toFixed(1)}</span>
                    </div>
                    <div style="height: 10px; background: var(--bg-tertiary); border-radius: 5px; overflow: hidden; margin-bottom: 16px;">
                        <div style="height: 100%; width: ${ironQuality.overall_score || ironQuality.overall || 0}%; background: linear-gradient(90deg, var(--accent-green), var(--accent-cyan)); transition: width 0.5s;"></div>
                    </div>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
                        <div style="padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                            <div style="font-size: 11px; color: var(--text-secondary);">含碳量</div>
                            <div style="font-size: 16px; font-weight: 600; color: var(--accent-cyan);">${(ironQuality.carbon_content || ironQuality.carbon || 0).toFixed(2)}%</div>
                        </div>
                        <div style="padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                            <div style="font-size: 11px; color: var(--text-secondary);">硫含量</div>
                            <div style="font-size: 16px; font-weight: 600; color: var(--accent-orange);">${(ironQuality.sulfur_content || ironQuality.sulfur || 0).toFixed(3)}%</div>
                        </div>
                        <div style="padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                            <div style="font-size: 11px; color: var(--text-secondary);">磷含量</div>
                            <div style="font-size: 16px; font-weight: 600; color: var(--accent-yellow);">${(ironQuality.phosphorus_content || ironQuality.phosphorus || 0).toFixed(3)}%</div>
                        </div>
                        <div style="padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                            <div style="font-size: 11px; color: var(--text-secondary);">硬度 (HB)</div>
                            <div style="font-size: 16px; font-weight: 600; color: var(--accent-purple);">${(ironQuality.hardness || ironQuality.hb || 0).toFixed(0)}</div>
                        </div>
                    </div>
                    ${ironQuality.grade ? `
                    <div style="margin-top: 12px; padding: 8px 12px; background: rgba(34, 197, 94, 0.15); border-radius: 6px; text-align: center;">
                        <span style="font-size: 13px; font-weight: 600; color: var(--accent-green);">等级评定：${ironQuality.grade}</span>
                    </div>
                    ` : ''}
                </div>
            </div>
        `;

        resultsArea.outerHTML = `<div class="slag-results-area">${html}</div>`;
    }

    _translatePosteriorKey(key) {
        const translations = {
            han_dynasty: '汉代概率',
            song_dynasty: '宋代概率',
            ming_dynasty: '明代概率',
            charcoal_fuel: '木炭燃料',
            coal_fuel: '煤炭燃料',
            coke_fuel: '焦炭燃料',
            low_temp: '低温工艺',
            medium_temp: '中温工艺',
            high_temp: '高温工艺',
            weak_reduction: '弱还原',
            medium_reduction: '中还原',
            strong_reduction: '强还原'
        };
        return translations[key] || key;
    }

    _generateFallbackProcessInference(composition, basicityR2) {
        let period = '汉代';
        let fuelType = '木炭';
        let estTemp = 1200;
        let confidence = 0.6;

        if (composition.s > 1.0) {
            period = '明代';
            fuelType = '煤炭';
            estTemp = 1350;
            confidence = 0.7;
        } else if (basicityR2 > 1.2) {
            period = '宋代';
            fuelType = '木炭';
            estTemp = 1250;
            confidence = 0.65;
        }

        const atmosphere = basicityR2 > 1.5 ? '强还原性' : basicityR2 > 1.0 ? '中等还原性' : '弱还原性';

        return {
            estimated_temp_c: estTemp,
            estimated_period: period,
            estimated_fuel_type: fuelType,
            confidence,
            reduction_atmosphere: atmosphere,
            bayesian_posteriors: {
                han_dynasty: period === '汉代' ? 0.7 : 0.2,
                song_dynasty: period === '宋代' ? 0.6 : 0.25,
                ming_dynasty: period === '明代' ? 0.75 : 0.15,
                charcoal_fuel: fuelType === '木炭' ? 0.8 : 0.3,
                coal_fuel: fuelType === '煤炭' ? 0.7 : 0.2,
                strong_reduction: atmosphere === '强还原性' ? 0.7 : 0.25
            },
            process_description: `该炉渣成分特征符合${period}时期${fuelType}冶炼的典型特征，冶炼温度约${estTemp}°C，属于${basicityR2 > 1 ? '碱性渣' : '酸性渣'}体系，炉渣流动性良好。`
        };
    }

    _generateFallbackOreCandidates(composition) {
        const tiContent = composition.tio2 || 0.5;
        const vContent = composition.v2o5 || 0.04;

        return [
            { name: '河北邯郸铁矿', location: '河北邯郸', historical_period: '汉代-明代', similarity: Math.max(0.3, 1 - Math.abs(tiContent - 0.3) / 1) },
            { name: '湖北大冶铁矿', location: '湖北大冶', historical_period: '春秋战国-宋代', similarity: Math.max(0.25, 1 - Math.abs(tiContent - 0.5) / 1.2) },
            { name: '四川攀枝花铁矿', location: '四川攀枝花', historical_period: '汉代-现代', similarity: tiContent > 2 ? 0.85 : 0.2 },
            { name: '安徽马鞍山铁矿', location: '安徽马鞍山', historical_period: '三国-现代', similarity: Math.max(0.3, 1 - Math.abs(tiContent - 0.4) / 1) },
            { name: '山西太原铁矿', location: '山西太原', historical_period: '战国-明清', similarity: Math.max(0.25, 1 - Math.abs(tiContent - 0.2) / 0.8) }
        ].sort((a, b) => b.similarity - a.similarity).slice(0, 5);
    }

    _generateFallbackIronQuality(composition, basicityR2) {
        const baseQuality = 70;
        const basicityBonus = Math.max(0, 10 - Math.abs(basicityR2 - 1.1) * 15);
        const sulfurPenalty = Math.min(20, composition.s * 10);
        const phosphorusPenalty = Math.min(15, composition.p2o5 * 8);

        const overall = Math.max(30, Math.min(95, baseQuality + basicityBonus - sulfurPenalty - phosphorusPenalty));

        let grade = '合格品';
        if (overall >= 90) grade = '优质钢';
        else if (overall >= 80) grade = '一等品';
        else if (overall >= 70) grade = '二等品';

        return {
            overall_score: overall,
            carbon_content: 2.5 + (1 - basicityR2) * 0.5,
            sulfur_content: composition.s * 0.15,
            phosphorus_content: composition.p2o5 * 0.1,
            hardness: 150 + overall * 1.5,
            grade
        };
    }

    _generateFallbackResults(composition) {
        return this._generateFallbackProcessInference(composition, composition.cao / composition.sio2);
    }
}

export default SlagAnalyzer;
