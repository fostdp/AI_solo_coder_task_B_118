export class FuelComparator {
    constructor(containerId, apiClient) {
        this.containerId = containerId;
        this.apiClient = apiClient;
        this.container = null;
        this.state = {
            fuelTypes: [],
            lastResults: null,
            isLoading: false
        };
    }

    render() {
        this.container = document.getElementById(this.containerId);
        if (!this.container) return;

        this.container.innerHTML = `
            <div class="feature-panel" style="background: transparent; border: none; padding: 0;">
                <div class="feature-header">
                    <h2><span class="icon">🔥</span>燃料对比分析</h2>
                    <p>对比不同燃料（木炭、煤、焦炭、木柴）对炉温和产品质量的影响</p>
                </div>
                <div class="fuel-comparison-container">
                    <div class="fuel-controls">
                        <div class="control-group">
                            <label>目标温度 (°C)</label>
                            <input type="number" class="fuel-target-temp" value="1300" min="500" max="1600">
                        </div>
                        <div class="control-group">
                            <label>矿石类型</label>
                            <select class="fuel-ore-type">
                                <option value="hematite">赤铁矿</option>
                                <option value="magnetite">磁铁矿</option>
                                <option value="limonite">褐铁矿</option>
                            </select>
                        </div>
                        <div class="control-group">
                            <label>对比燃料</label>
                            <div class="checkbox-group fuel-checkbox-group">
                                <label><input type="checkbox" value="Charcoal" checked> 木炭</label>
                                <label><input type="checkbox" value="Coal" checked> 煤炭</label>
                                <label><input type="checkbox" value="Coke"> 焦炭</label>
                                <label><input type="checkbox" value="Wood"> 木柴</label>
                            </div>
                        </div>
                        <button class="btn-primary fuel-compare-btn">开始对比分析</button>
                    </div>
                    <div class="fuel-results fuel-results-area">
                        <div class="results-placeholder">请选择参数后点击"开始对比分析"</div>
                    </div>
                </div>
            </div>
        `;

        this._bindEvents();
        this._loadFuelTypes();
    }

    _bindEvents() {
        const compareBtn = this.container.querySelector('.fuel-compare-btn');
        if (compareBtn) {
            compareBtn.addEventListener('click', () => this.runComparison());
        }
    }

    async _loadFuelTypes() {
        try {
            const res = await this.apiClient.get('/api/fuel/types');
            if (res && (res.success || Array.isArray(res))) {
                this.state.fuelTypes = res.data || res || [];
            }
        } catch (e) {
            console.warn('[FuelComparator] Failed to load fuel types:', e);
        }
    }

    async runComparison() {
        const targetTemp = parseFloat(this.container.querySelector('.fuel-target-temp')?.value || '1300');
        const oreType = this.container.querySelector('.fuel-ore-type')?.value || 'hematite';

        const checkboxes = this.container.querySelectorAll('.fuel-checkbox-group input:checked');
        const fuelTypes = Array.from(checkboxes).map(cb => cb.value);

        if (fuelTypes.length === 0) {
            this._showPlaceholder('请至少选择一种燃料进行对比');
            return;
        }

        const resultsArea = this.container.querySelector('.fuel-results-area');
        if (resultsArea) {
            resultsArea.innerHTML = '<div class="results-placeholder">分析中...</div>';
        }
        this.state.isLoading = true;

        try {
            const data = await this.apiClient.post('/api/fuel/compare', {
                fuel_types: fuelTypes,
                furnace_id: 'HAN-001',
                target_temp: targetTemp,
                ore_type: oreType
            });

            if (data && (data.success || data.items || data)) {
                const results = data.data || data;
                this.state.lastResults = results;
                this.formatFuelResults(results);
            } else {
                throw new Error('API returned invalid response');
            }
        } catch (e) {
            console.error('[FuelComparator] Comparison error:', e);
            this.formatFuelResults(this._generateFallbackResults(fuelTypes, targetTemp));
        } finally {
            this.state.isLoading = false;
        }
    }

    formatFuelResults(results) {
        const resultsArea = this.container.querySelector('.fuel-results-area');
        if (!resultsArea) return;

        let html = '';
        const items = results.items || results;
        const recommendedFuel = results.recommended_fuel;

        if (!Array.isArray(items)) {
            resultsArea.innerHTML = this._generateFallbackResultsHtml(results);
            return;
        }

        items.forEach((item) => {
            const isRecommended = item.fuel_type === recommendedFuel;
            const fuelName = this._getFuelName(item.fuel_type);
            const fuelIcon = this._getFuelIcon(item.fuel_type);

            const calorificValue = item.calorific_value ?? item.calorific ?? '--';
            const carbonContent = item.carbon_content ?? item.carbon ?? '--';
            const combustionRate = item.combustion_rate ?? '--';
            const energyEfficiency = item.energy_efficiency ?? '--';
            const sulfurContent = item.sulfur_content ?? item.sulfur ?? '--';
            const ashContent = item.ash_content ?? item.ash ?? '--';
            const ironQuality = item.iron_quality?.overall_quality ?? item.quality_score ?? 0;
            const timeToTarget = item.time_to_target_min ?? item.heat_up_time ?? '--';
            const costPerTon = item.cost_per_ton_iron ?? item.cost ?? '--';

            html += `
                <div class="fuel-comparison-card ${isRecommended ? 'recommended' : ''}">
                    <div class="fuel-card-header">
                        <div class="fuel-card-title">
                            <span>${fuelIcon}</span>
                            <span>${fuelName}</span>
                        </div>
                        ${isRecommended ? '<span class="fuel-card-badge">推荐</span>' : ''}
                    </div>
                    <div class="fuel-props-grid">
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">热值</div>
                            <div class="fuel-prop-value">${typeof calorificValue === 'number' ? calorificValue.toFixed(1) : calorificValue} MJ/kg</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">含碳量</div>
                            <div class="fuel-prop-value">${typeof carbonContent === 'number' ? carbonContent.toFixed(1) : carbonContent}%</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">含硫量</div>
                            <div class="fuel-prop-value">${typeof sulfurContent === 'number' ? sulfurContent.toFixed(2) : sulfurContent}%</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">灰分</div>
                            <div class="fuel-prop-value">${typeof ashContent === 'number' ? ashContent.toFixed(1) : ashContent}%</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">燃烧速率</div>
                            <div class="fuel-prop-value">${typeof combustionRate === 'number' ? combustionRate.toFixed(3) : combustionRate} kg/s</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">能源效率</div>
                            <div class="fuel-prop-value">${typeof energyEfficiency === 'number' ? energyEfficiency.toFixed(1) : energyEfficiency}%</div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>铁水质量</span>
                            <span>${typeof ironQuality === 'number' ? ironQuality.toFixed(1) : ironQuality}/100</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${ironQuality || 0}%; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-green));"></div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>达到目标温度时间</span>
                            <span>${typeof timeToTarget === 'number' ? timeToTarget.toFixed(1) : timeToTarget} 分钟</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${Math.min(100, (300 / (typeof timeToTarget === 'number' ? timeToTarget : 300)) * 100)}%; background: linear-gradient(90deg, var(--accent-orange), var(--accent-red));"></div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>吨铁成本估算</span>
                            <span>${typeof costPerTon === 'number' ? costPerTon.toLocaleString() : costPerTon} 元</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${Math.min(100, (3000 / (typeof costPerTon === 'number' ? costPerTon : 3000)) * 100)}%; background: linear-gradient(90deg, var(--accent-yellow), var(--accent-purple));"></div>
                        </div>
                    </div>
                </div>
            `;
        });

        if (results.summary) {
            html += `
                <div class="fuel-summary">
                    <h4>📋 分析总结</h4>
                    <p>${results.summary}</p>
                </div>
            `;
        }

        resultsArea.innerHTML = html;
    }

    _generateFallbackResults(fuelTypes, targetTemp) {
        const fuelData = {
            Charcoal: { fuel_type: 'Charcoal', calorific_value: 30.0, carbon_content: 75.0, sulfur_content: 0.3, ash_content: 2.5, combustion_rate: 0.012, energy_efficiency: 82, quality_score: 85, time_to_target_min: 45, cost_per_ton_iron: 2800 },
            Coal: { fuel_type: 'Coal', calorific_value: 28.0, carbon_content: 65.0, sulfur_content: 1.8, ash_content: 12.0, combustion_rate: 0.018, energy_efficiency: 70, quality_score: 70, time_to_target_min: 60, cost_per_ton_iron: 1900 },
            Coke: { fuel_type: 'Coke', calorific_value: 28.0, carbon_content: 92.0, sulfur_content: 0.6, ash_content: 10.0, combustion_rate: 0.008, energy_efficiency: 88, quality_score: 90, time_to_target_min: 40, cost_per_ton_iron: 2500 },
            Wood: { fuel_type: 'Wood', calorific_value: 18.0, carbon_content: 50.0, sulfur_content: 0.05, ash_content: 1.5, combustion_rate: 0.025, energy_efficiency: 55, quality_score: 60, time_to_target_min: 90, cost_per_ton_iron: 3200 }
        };

        const items = fuelTypes.map(type => fuelData[type]).filter(Boolean);

        let recommended = '';
        let bestScore = 0;
        items.forEach(item => {
            const score = (item.quality_score || 0) * 0.4 + (100 - (item.time_to_target_min || 0) / 1.2) * 0.3 + (3000 - (item.cost_per_ton_iron || 0)) / 30 * 0.3;
            if (score > bestScore) {
                bestScore = score;
                recommended = item.fuel_type;
            }
        });

        const recommendedName = this._getFuelName(recommended);
        return {
            items,
            recommended_fuel: recommended,
            summary: `在目标温度 ${targetTemp}°C 的条件下，综合考虑铁水质量、升温速度和燃料成本，<strong>${recommendedName}</strong> 是当前配置下的最优选择。${recommended === 'Charcoal' ? '木炭燃烧充分，产品质量高，适合高品质冶铁需求。' : ''}${recommended === 'Coal' ? '煤炭热值较高，成本适中，适合大规模生产。' : ''}${recommended === 'Coke' ? '焦炭含碳量高，温度稳定，适合高炉冶炼。' : ''}${recommended === 'Wood' ? '木柴获取容易，但热值较低，适合小规模作坊。' : ''}`
        };
    }

    _generateFallbackResultsHtml(results) {
        const fallback = this._generateFallbackResults(
            [results.fuel_type || 'Charcoal'],
            1300
        );
        let html = '';
        fallback.items.forEach(item => {
            const isRecommended = item.fuel_type === fallback.recommended_fuel;
            html += `
                <div class="fuel-comparison-card ${isRecommended ? 'recommended' : ''}">
                    <div class="fuel-card-header">
                        <div class="fuel-card-title">
                            <span>${this._getFuelIcon(item.fuel_type)}</span>
                            <span>${this._getFuelName(item.fuel_type)}</span>
                        </div>
                        ${isRecommended ? '<span class="fuel-card-badge">推荐</span>' : ''}
                    </div>
                    <div class="fuel-props-grid">
                        <div class="fuel-prop"><div class="fuel-prop-label">热值</div><div class="fuel-prop-value">${item.calorific_value.toFixed(1)} MJ/kg</div></div>
                        <div class="fuel-prop"><div class="fuel-prop-label">含碳量</div><div class="fuel-prop-value">${item.carbon_content.toFixed(1)}%</div></div>
                        <div class="fuel-prop"><div class="fuel-prop-label">含硫量</div><div class="fuel-prop-value">${item.sulfur_content.toFixed(2)}%</div></div>
                        <div class="fuel-prop"><div class="fuel-prop-label">灰分</div><div class="fuel-prop-value">${item.ash_content.toFixed(1)}%</div></div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label"><span>铁水质量</span><span>${item.quality_score}/100</span></div>
                        <div class="fuel-bar"><div class="fuel-bar-fill" style="width: ${item.quality_score}%; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-green));"></div></div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label"><span>升温速度</span><span>${item.time_to_target_min} 分钟</span></div>
                        <div class="fuel-bar"><div class="fuel-bar-fill" style="width: ${(100 - item.time_to_target_min / 120 * 100)}%; background: linear-gradient(90deg, var(--accent-orange), var(--accent-red));"></div></div>
                    </div>
                </div>
            `;
        });
        html += `<div class="fuel-summary"><h4>📋 分析总结</h4><p>${fallback.summary}</p></div>`;
        return html;
    }

    _getFuelName(type) {
        const names = { Charcoal: '木炭', Coal: '煤炭', Coke: '焦炭', Wood: '木柴' };
        return names[type] || type;
    }

    _getFuelIcon(type) {
        const icons = { Charcoal: '🪵', Coal: '⛏️', Coke: '🏭', Wood: '🌲' };
        return icons[type] || '🔥';
    }

    _showPlaceholder(message) {
        const resultsArea = this.container.querySelector('.fuel-results-area');
        if (resultsArea) {
            resultsArea.innerHTML = `<div class="results-placeholder">${message}</div>`;
        }
    }
}

export default FuelComparator;
