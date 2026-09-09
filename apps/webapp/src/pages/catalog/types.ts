export interface ServiceRateFormValues {
	label: string
	/** One of `ServiceRateUnit`'s built-in codes, or an organization's own
	 * custom unit code (#449) — a plain string because a fixed union cannot
	 * name a value it does not know about yet. */
	unit: string
	rate: string
	description: string
}
