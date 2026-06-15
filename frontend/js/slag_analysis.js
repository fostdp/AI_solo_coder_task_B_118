export class SlagAnalysis {
    constructor(apiBase) {
        this.apiBase = apiBase;
        this.oreSources = [];
        this.init();
    }

    init() {
        this.bindEvents();
        this.loadOreSources();
    }

    bindEvents() {
        const btnAnalyze = document.getElementById('btnAnalyzeSlag');
        if (btnAnalyze) {
            btnAnalyze.addEventListener('click', () => this.handleAnalyze());
        }

        const btnGenerate = document.getElementById('btnGenerateSample');
        if (btnGenerate) {
            btnGenerate.addEventListener('click', () => this.handleGenerateSample());
        }
    }

    async loadOreSources() {
        try {
            const res = await fetch(`${this.apiBase}/api/slag/ore-sources`);
            if (res.ok) {
                this.oreSources = await res.json();
            }
        } catch (e) {
            console.warn('Failed to load ore sources:', e);
        }
    }

    getInputValues() {
        const fields = [
            'sio2', 'cao', 'mgo', 'al2o3', 'feo', 'fe2o3',
            'mno', 'p2o5', 's', 'tio2', 'v2o5', 'cr2o3'
        ];
        const values = {};
        fields.forEach(f => {
            const el = document.getElementById('slag' + f.charAt(0).toUpperCase() + f.slice(1));
            values[f] = parseFloat(el?.value || '0');
        });
        return values;
    }

    async handleAnalyze() {
        const composition = this.getInputValues();
        const resultsDiv = document.getElementById('slagResults');
        
        if (resultsDiv) {
            resultsDiv.innerHTML = '<div class="results-placeholder">分析中...</div>';
        }

        try {
            const res = await fetch(`${this.apiBase}/api/slag/analyze`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    composition,
                    furnace_id: 'HAN-001',
                    sample_id: 'sample-' + Date.now()
                })
            });

            if (res.ok) {
                const data = await res.json();
                this.renderResults(data);
            } else {
                throw new Error('API request failed');
            }
        } catch (e) {
            console.error('Slag analysis error:', e);
            if (resultsDiv) {
                resultsDiv.innerHTML = this.renderFallbackResults(composition);
            }
        }
    }

    async handleGenerateSample() {
        try {
            const res = await fetch(`${this.apiBase}/api/slag/sample?furnace_id=HAN-001&fuel_type=Charcoal&ore_type=hematite`);
            if (res.ok) {
                const data = await res.json();
                this.fillFormValues(data.composition || data);
            } else {
                this.fillFormValues(this.generateFallbackSample());
            }
        } catch (e) {
            this.fillFormValues(this.generateFallbackSample());
        }
    }

    fillFormValues(composition) {
        const mapping = {
            sio2: 'slagSio2', cao: 'slagCao', mgo: 'slagMgo',
            al2o3: 'slagAl2o3', feo: 'slagFeo', fe2o3: 'slagFe2o3',
            mno: 'slagMno', p2o5: 'slagP2o5', s: 'slagS',
            tio2: 'slagTio2', v2o5: 'slagV2o5', cr2o3: 'slagCr2o3'
        };

        Object.entries(mapping).forEach(([key, elId]) => {
            const el = document.getElementById(elId);
            if (el && composition[key] !== undefined) {
                el.value = composition[key].toFixed(2);
            }
        });
    }

    generateFallbackSample() {
        return {
            sio2: 35.2, cao: 14.8, mgo: 7.5, al2o3: 9.6,
            feo: 12.3, fe2o3: 3.2, mno: 1.8, p2o5: 0.45,
            s: 0.78, tio2: 0.55, v2o5: 0.04, cr2o3: 0.025
        };
    }

    renderResults(data) {
        const resultsDiv = document.getElementById('slagResults');
        if (!resultsDiv) return;

        let html = '';

        if (data.basicity !== undefined) {
            html += `
                <div class="slag-result-section">
                    <h4>📊 炉渣基本属性</h4>
                    <div class="slag-basicity-info">
                        <div class="slag-info-item">
                            <div class="slag-info-value">${data.basicity?.toFixed(2) || '--'}</div>
                            <div class="slag-info-label">二元碱度 R2</div>
                        </div>
                        <div class="slag-info-item">
                            <div class="slag-info-value">${data.melting_point?.toFixed(0) || '--'}°C</div>
                            <div class="slag-info-label">估算熔点</div>
                        </div>
                        <div class="slag-info-item">
                            <div class="slag-info-value">${data.viscosity?.toFixed(2) || '--'}</div>
                            <div class="slag-info-label">粘度 Pa·s</div>
                        </div>
                    </div>
                </div>
            `;
        }

        if (data.process_inference) {
            const pi = data.process_inference;
            html += `
                <div class="slag-result-section">
                    <h4>🏺 工艺反推</h4>
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
                    <div style="margin-top: 10px; padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                        <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">还原气氛</div>
                        <div style="font-size: 14px; font-weight: 600; color: var(--accent-orange);">${pi.reduction_atmosphere || '--'}
                            <span style="font-size: 11px; color: var(--text-muted);">(置信度 ${(pi.confidence * 100).toFixed(0)}%)</span>
                        </div>
                    </div>
                    ${pi.process_description ? `
                    <div style="margin-top: 10px; padding: 10px; background: rgba(167, 139, 250, 0.1); border-radius: 8px; border-left: 3px solid var(--accent-purple);">
                        <div style="font-size: 12px; color: var(--text-secondary);">${pi.process_description}</div>
                    </div>` : ''}
                </div>
            `;
        }

        if (data.ore_source_candidates && data.ore_source_candidates.length > 0) {
            html += `
                <div class="slag-result-section">
                    <h4>⛏️ 矿石来源匹配</h4>
                    <div class="ore-source-list">
                        ${data.ore_source_candidates.map(candidate => `
                            <div class="ore-source-item">
                                <div class="ore-source-info">
                                    <div class="ore-source-name">${candidate.name || candidate.source_name || '未知矿源'}</div>
                                    <div class="ore-source-loc">${candidate.location || ''} · ${candidate.historical_period || ''}</div>
                                </div>
                                <div class="ore-source-score">
                                    <div class="ore-source-percent">${(candidate.similarity * 100).toFixed(1)}%</div>
                                    <div class="ore-source-bar">
                                        <div class="ore-source-bar-fill" style="width: ${candidate.similarity * 100}%;"></div>
                                    </div>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        resultsDiv.innerHTML = html;
    }

    renderFallbackResults(composition) {
        const basicity = composition.cao / composition.sio2;
        const meltingPoint = 1350 + (composition.al2o3 - 10) * 15 - (composition.cao - 15) * 10;
        const viscosity = 0.5 + Math.abs(basicity - 1.0) * 0.8;

        let period = '汉代';
        let fuelType = '木炭';
        let estTemp = 1200;
        let confidence = 0.6;

        if (composition.s > 1.0) {
            period = '明代';
            fuelType = '煤炭';
            estTemp = 1350;
            confidence = 0.7;
        } else if (basicity > 1.2) {
            period = '宋代';
            fuelType = '木炭';
            estTemp = 1250;
            confidence = 0.65;
        }

        const tiContent = composition.tio2 || 0.5;
        const vContent = composition.v2o5 || 0.04;

        const candidates = [
            { name: '河北邯郸铁矿', location: '河北邯郸', period: '汉代-明代', sim: Math.max(0.3, 1 - Math.abs(tiContent - 0.3) / 1) },
            { name: '湖北大冶铁矿', location: '湖北大冶', period: '春秋战国-宋代', sim: Math.max(0.25, 1 - Math.abs(tiContent - 0.5) / 1.2) },
            { name: '四川攀枝花铁矿', location: '四川攀枝花', period: '汉代-现代', sim: tiContent > 2 ? 0.85 : 0.2 },
            { name: '安徽马鞍山铁矿', location: '安徽马鞍山', period: '三国-现代', sim: Math.max(0.3, 1 - Math.abs(tiContent - 0.4) / 1) },
            { name: '山西太原铁矿', location: '山西太原', period: '战国-明清', sim: Math.max(0.25, 1 - Math.abs(tiContent - 0.2) / 0.8) }
        ].sort((a, b) => b.sim - a.sim).slice(0, 5);

        return `
            <div class="slag-result-section">
                <h4>📊 炉渣基本属性</h4>
                <div class="slag-basicity-info">
                    <div class="slag-info-item">
                        <div class="slag-info-value">${basicity.toFixed(2)}</div>
                        <div class="slag-info-label">二元碱度 R2</div>
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
            </div>

            <div class="slag-result-section">
                <h4>🏺 工艺反推</h4>
                <div class="slag-basicity-info">
                    <div class="slag-info-item">
                        <div class="slag-info-value" style="font-size: 14px;">${estTemp}°C</div>
                        <div class="slag-info-label">推断冶炼温度</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value" style="font-size: 14px;">${period}</div>
                        <div class="slag-info-label">推断历史时期</div>
                    </div>
                    <div class="slag-info-item">
                        <div class="slag-info-value" style="font-size: 14px;">${fuelType}</div>
                        <div class="slag-info-label">推断燃料类型</div>
                    </div>
                </div>
                <div style="margin-top: 10px; padding: 10px; background: var(--bg-secondary); border-radius: 8px;">
                    <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 4px;">还原气氛</div>
                    <div style="font-size: 14px; font-weight: 600; color: var(--accent-orange);">
                        ${basicity > 1.5 ? '强还原性' : basicity > 1.0 ? '中等还原性' : '弱还原性'}
                        <span style="font-size: 11px; color: var(--text-muted);">(置信度 ${(confidence * 100).toFixed(0)}%)</span>
                    </div>
                </div>
                <div style="margin-top: 10px; padding: 10px; background: rgba(167, 139, 250, 0.1); border-radius: 8px; border-left: 3px solid var(--accent-purple);">
                    <div style="font-size: 12px; color: var(--text-secondary);">
                        该炉渣成分特征符合${period}时期${fuelType}冶炼的典型特征，
                        冶炼温度约${estTemp}°C，属于${basicity > 1 ? '碱性渣' : '酸性渣'}体系，
                        炉渣流动性${viscosity < 0.5 ? '良好' : viscosity < 1.0 ? '一般' : '较差'}。
                    </div>
                </div>
            </div>

            <div class="slag-result-section">
                <h4>⛏️ 矿石来源匹配</h4>
                <div class="ore-source-list">
                    ${candidates.map(c => `
                        <div class="ore-source-item">
                            <div class="ore-source-info">
                                <div class="ore-source-name">${c.name}</div>
                                <div class="ore-source-loc">${c.location} · ${c.period}</div>
                            </div>
                            <div class="ore-source-score">
                                <div class="ore-source-percent">${(c.sim * 100).toFixed(1)}%</div>
                                <div class="ore-source-bar">
                                    <div class="ore-source-bar-fill" style="width: ${c.sim * 100}%;"></div>
                                </div>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }
}
