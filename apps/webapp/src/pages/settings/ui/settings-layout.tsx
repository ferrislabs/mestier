import type { SettingsNavGroup } from '#/pages/settings/nav'
import type { SettingsSection } from '#/pages/settings/types'
import { AnchorNav } from '#/pages/settings/ui/anchor-nav'

interface SettingsLayoutProps {
	groups: SettingsNavGroup[]
	sections: SettingsSection[]
	activeId: string
}

export function SettingsLayout({
	groups,
	sections,
	activeId,
}: SettingsLayoutProps) {
	return (
		<div className="grid gap-6 lg:grid-cols-[13rem_minmax(0,1fr)] lg:gap-10">
			<AnchorNav groups={groups} activeId={activeId} />
			<div className="grid min-w-0 content-start gap-6">
				{sections.map((section) => {
					const Section = section.Component
					return (
						<section
							key={section.id}
							id={section.id}
							className="scroll-mt-6 outline-none"
						>
							<Section />
						</section>
					)
				})}
			</div>
		</div>
	)
}
