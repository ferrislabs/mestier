export const SETTINGS_SECTIONS: readonly { id: string; label: string }[] = [
	{ id: 'entreprise', label: 'Entreprise' },
	{ id: 'facturation', label: 'Facturation' },
	{ id: 'numerotation', label: 'Numérotation' },
	{ id: 'stripe', label: 'Stripe' },
]

function goTo(id: string) {
	const section = document.getElementById(id)
	if (!section) return
	const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
	section.scrollIntoView({
		block: 'start',
		behavior: reduced ? 'auto' : 'smooth',
	})
	section.focus({ preventScroll: true })
}

export function SettingsNav() {
	return (
		<nav aria-label="Sommaire des réglages" className="lg:sticky lg:top-6">
			<ul className="flex flex-wrap gap-1 lg:grid">
				{SETTINGS_SECTIONS.map((section) => (
					<li key={section.id}>
						<a
							href={`#${section.id}`}
							onClick={(event) => {
								event.preventDefault()
								goTo(section.id)
							}}
							className="block rounded-md px-3 py-1.5 text-[13px] text-muted-foreground outline-none transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
						>
							{section.label}
						</a>
					</li>
				))}
			</ul>
		</nav>
	)
}
