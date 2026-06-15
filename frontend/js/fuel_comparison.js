export class FuelComparison {
    constructor(apiBase) {
        this.apiBase = apiBase;
        this.fuelTypes = [];
        this.init();
    }

    init() {
        this.bindEvents();
        this.loadFuelTypes();
    }

    bindEvents() {
        const btnCompare = document.getElementById('btnCompareFuels');
        if (btnCompare) {
            btnCompare.addEventListener('click', () => this.handleCompare());
        }
    }

    async loadFuelTypes() {
        try {
            const res = await fetch(`${this.apiBase}/api/fuel/types`);
            if (res.ok) {
                this.fuelTypes = await res.json();
            }
        } catch (e) {
            console.warn('Failed to load fuel types:', e);
        }
    }

    async handleCompare() {
        const targetTemp = parseFloat(document.getElementById('fuelTargetTemp')?.value || '1300');
        const oreType = document.getElementById('fuelOreType')?.value || 'hematite';
        
        const checkboxes = document.querySelectorAll('#fuelCheckboxGroup input:checked');
        const fuelTypes = Array.from(checkboxes).map(cb => cb.value);

        if (fuelTypes.length === 0) {
            alert('请至少选择一种燃料进行对比');
            return;
        }

        const resultsDiv = document.getElementById('fuelResults');
        if (resultsDiv) {
            resultsDiv.innerHTML = '<div class="results-placeholder">分析中...</div>';
        }

        try {
            const res = await fetch(`${this.apiBase}/api/fuel/compare`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    fuel_types: fuelTypes,
                    furnace_id: 'HAN-001',
                    target_temp: targetTemp,
                    ore_type: oreType
                })
            });

            if (res.ok) {
                const data = await res.json();
                this.renderResults(data);
            } else {
                throw new Error('API request failed');
            }
        } catch (e) {
            console.error('Fuel comparison error:', e);
            if (resultsDiv) {
                resultsDiv.innerHTML = this.renderFallbackResults(fuelTypes, targetTemp);
            }
        }
    }

    renderResults(data) {
        const resultsDiv = document.getElementById('fuelResults');
        if (!resultsDiv) return;

        let html = '';

        data.items?.forEach((item, idx) => {
            const isRecommended = item.fuel_type === data.recommended_fuel;
            const fuelName = this.getFuelName(item.fuel_type);
            const fuelIcon = this.getFuelIcon(item.fuel_type);

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
                            <div class="fuel-prop-value">${item.calorific_value?.toFixed(1) || '--'} MJ/kg</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">含碳量</div>
                            <div class="fuel-prop-value">${item.carbon_content?.toFixed(1) || '--'}%</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">燃烧速率</div>
                            <div class="fuel-prop-value">${item.combustion_rate?.toFixed(3) || '--'} kg/s</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">能源效率</div>
                            <div class="fuel-prop-value">${item.energy_efficiency?.toFixed(1) || '--'}%</div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>铁水质量</span>
                            <span>${item.iron_quality?.overall_quality?.toFixed(1) || '--'}/100</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${item.iron_quality?.overall_quality || 0}%; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-green));"></div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>达到目标温度时间</span>
                            <span>${item.time_to_target_min?.toFixed(1) || '--'} 分钟</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${Math.min(100, 300 / (item.time_to_target_min || 300) * 100)}%; background: linear-gradient(90deg, var(--accent-orange), var(--accent-red));"></div>
                        </div>
                    </div>
                </div>
            `;
        });

        if (data.summary) {
            html += `
                <div class="fuel-summary">
                    <h4>📋 分析总结</h4>
                    <p>${data.summary}</p>
                </div>
            `;
        }

        resultsDiv.innerHTML = html;
    }

    renderFallbackResults(fuelTypes, targetTemp) {
        const fuelData = {
            Charcoal: { name: '木炭', icon: '🪵', calorific: 30.0, carbon: 75.0, quality: 85, time: 45 },
            Coal: { name: '煤炭', icon: '⛏️', calorific: 28.0, carbon: 65.0, quality: 70, time: 60 },
            Coke: { name: '焦炭', icon: '🏭', calorific: 28.0, carbon: 92.0, quality: 90, time: 40 },
            Wood: { name: '木柴', icon: '🌲', calorific: 18.0, carbon: 50.0, quality: 60, time: 90 }
        };

        let html = '';
        let recommended = '';
        let bestScore = 0;

        fuelTypes.forEach(type => {
            const d = fuelData[type];
            if (!d) return;
            const score = d.quality * 0.6 + (100 - d.time) * 0.4;
            if (score > bestScore) {
                bestScore = score;
                recommended = type;
            }
        });

        fuelTypes.forEach(type => {
            const d = fuelData[type];
            if (!d) return;
            const isRec = type === recommended;

            html += `
                <div class="fuel-comparison-card ${isRec ? 'recommended' : ''}">
                    <div class="fuel-card-header">
                        <div class="fuel-card-title">
                            <span>${d.icon}</span>
                            <span>${d.name}</span>
                        </div>
                        ${isRec ? '<span class="fuel-card-badge">推荐</span>' : ''}
                    </div>
                    <div class="fuel-props-grid">
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">热值</div>
                            <div class="fuel-prop-value">${d.calorific} MJ/kg</div>
                        </div>
                        <div class="fuel-prop">
                            <div class="fuel-prop-label">含碳量</div>
                            <div class="fuel-prop-value">${d.carbon}%</div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>铁水质量</span>
                            <span>${d.quality}/100</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${d.quality}%; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-green));"></div>
                        </div>
                    </div>
                    <div class="fuel-bar-row">
                        <div class="fuel-bar-label">
                            <span>升温速度</span>
                            <span>${d.time} 分钟</span>
                        </div>
                        <div class="fuel-bar">
                            <div class="fuel-bar-fill" style="width: ${(100 - d.time / 120 * 100)}%; background: linear-gradient(90deg, var(--accent-orange), var(--accent-red));"></div>
                        </div>
                    </div>
                </div>
            `;
        });

        html += `
            <div class="fuel-summary">
                <h4>📋 分析总结</h4>
                <p>在目标温度 ${targetTemp}°C 的条件下，综合考虑铁水质量、升温速度和燃料成本，
                <strong>${fuelData[recommended]?.name || '木炭'}</strong> 是当前配置下的最优选择。
                ${recommended === 'Charcoal' ? '木炭燃烧充分，产品质量高，适合高品质冶铁需求。' : ''}
                ${recommended === 'Coal' ? '煤炭热值较高，成本适中，适合大规模生产。' : ''}
                ${recommended === 'Coke' ? '焦炭含碳量高，温度稳定，适合高炉冶炼。' : ''}
                ${recommended === 'Wood' ? '木柴获取容易，但热值较低，适合小规模作坊。' : ''}</p>
            </div>
        `;

        return html;
    }

    getFuelName(type) {
        const names = { Charcoal: '木炭', Coal: '煤炭', Coke: '焦炭', Wood: '木柴' };
        return names[type] || type;
    }

    getFuelIcon(type) {
        const icons = { Charcoal: '🪵', Coal: '⛏️', Coke: '🏭', Wood: '🌲' };
        return icons[type] || '🔥';
    }
}
