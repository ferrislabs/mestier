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
		<div className="flex gap-8">
			<AnchorNav groups={groups} activeId={activeId} />
			<div className="flex min-w-0 flex-1 flex-col gap-10">
				{sections.map((section) => {
					const Section = section.Component
					return (
						<section
							key={section.id}
							id={section.id}
							aria-label={section.label}
							className="scroll-mt-20"
						>
							<Section />
						</section>
					)
				})}
			</div>
		</div>
	)
}
