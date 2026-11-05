import { AlertCircle } from 'lucide-react'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { INFO_TAB, PLAN_TAB, type Rubrique } from '#/pages/doe/lib/rubriques'
import type { DoeForm } from '#/pages/doe/types'
import { DoeInfoTab } from '#/pages/doe/ui/doe-info-tab'
import { DoePlanTab } from '#/pages/doe/ui/doe-plan-tab'
import {
	DoeSectionTab,
	type SectionActions,
} from '#/pages/doe/ui/doe-section-tab'
import { DoeSidebar, type DoeSidebarProps } from '#/pages/doe/ui/doe-sidebar'

type InfoProps = Parameters<typeof DoeInfoTab>[0]
type PlanProps = Parameters<typeof DoePlanTab>[0]

export { INFO_TAB, PLAN_TAB }

interface DoeEditorUIProps {
	sidebar: Omit<DoeSidebarProps, 'items' | 'selected'>
	form: DoeForm
	items: readonly Rubrique[]
	activeTab: string
	error: string | null
	fileUrls: Record<string, string | undefined>
	info: Omit<InfoProps, 'form' | 'fileUrls'>
	plan: Omit<PlanProps, 'sections'>
	sectionActions: SectionActions
}

export function DoeEditorUI({
	sidebar,
	form,
	items,
	activeTab,
	error,
	fileUrls,
	info,
	plan,
	sectionActions,
}: DoeEditorUIProps) {
	const current = items.find((item) => item.id === activeTab)

	const content = () => {
		if (current?.section) {
			return (
				<DoeSectionTab
					section={current.section}
					number={current.number ?? 1}
					fileUrls={fileUrls}
					actions={sectionActions}
				/>
			)
		}
		if (activeTab === PLAN_TAB) {
			return <DoePlanTab sections={form.sections} {...plan} />
		}
		return <DoeInfoTab form={form} fileUrls={fileUrls} {...info} />
	}

	return (
		<div className="h-full min-h-0 overflow-y-auto lg:grid lg:grid-cols-[minmax(20rem,24rem)_minmax(0,1fr)] lg:grid-rows-[minmax(0,1fr)] lg:overflow-hidden">
			<DoeSidebar {...sidebar} items={items} selected={activeTab} />
			<section aria-label="Rubrique du DOE" className="min-w-0 lg:min-h-0">
				<RubriqueLayout
					key={activeTab}
					title={current?.title ?? 'Informations'}
					description={current?.description ?? ''}
					wide
				>
					{error ? (
						<div
							role="alert"
							className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
						>
							<AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
							{error}
						</div>
					) : null}
					{content()}
				</RubriqueLayout>
			</section>
		</div>
	)
}
