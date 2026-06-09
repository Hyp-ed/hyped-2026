module.exports = {
	rollup(config) {
		// Mark telemetry-types as external so it doesn't try to bundle it
		const external = config.external || [];
		config.external = [
			...(Array.isArray(external) ? external : [external].filter(Boolean)),
			'@hyped/telemetry-types',
		];
		return config;
	},
};