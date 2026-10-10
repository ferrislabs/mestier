import { AlertCircle } from 'lucide-react'
import { Badge } from '#/components/ui/badge'
import { PageShell } from '#/components/ui/surface'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { enabledNumberedSections, tabLabel } from '#/pages/doe/lib/sections'
import type { DoeForm } from '#/pages/doe/types'
import { DoeHeader } from '#/pages/doe/ui/doe-header'
import { DoeInfoTab } from '#/pages/doe/ui/doe-info-tab'
import { DoePlanTab } from '#/pages/doe/ui/doe-plan-tab'
import {
	DoeSectionTab,
	type SectionActions,
	sectionItemCount,
} from '#/pages/doe/ui/doe-section-tab'

export const INFO_TAB = 'info'
export const PLAN_TAB = 'plan'

type HeaderProps = Parameters<typeof DoeHeader>[0]
type InfoProps = Parameters<typeof DoeInfoTab>[0]
type PlanProps = Parameters<typeof DoePlanTab>[0]

interface DoeEditorUIProps {
	header: HeaderProps
	form: DoeForm
	activeTab: string
	error: string | null
	fileUrls: Record<string, string | undefined>
	info: Omit<InfoProps, 'form' | 'fileUrls'>
	plan: Omit<PlanProps, 'sections'>
	sectionActions: SectionActions
	onTabChange: (tab: string) => void
}

export function DoeEditorUI({
	header,
	form,
	activeTab,
	error,
	fileUrls,
	info,
	plan,
	sectionActions,
	onTabChange,
}: DoeEditorUIProps) {
	const enabled = enabledNumberedSections(form.sections)

	return (
		<PageShell>
			<DoeHeader {...header} />
			{error ? (
				<div
					role="alert"
					className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive"
				>
					<AlertCircle className="mt-0.5 size-4 shrink-0" />
					{error}
				</div>
			) : null}
			<Tabs value={activeTab} onValueChange={onTabChange}>
				<TabsList
					variant="line"
					className="h-auto w-full justify-start overflow-x-auto overflow-y-hidden pb-2"
				>
					<TabsTrigger value={INFO_TAB} className="flex-none">
						Informations
					</TabsTrigger>
					<TabsTrigger value={PLAN_TAB} className="flex-none">
						Plan du DOE
					</TabsTrigger>
					{enabled.map(({ section, number }) => (
						<TabsTrigger
							key={section.id}
							value={section.id}
							title={`${number}. ${section.title}`}
							className="flex-none"
						>
							{number}. {tabLabel(section.title)}
							<Badge variant="secondary">{sectionItemCount(section)}</Badge>
						</TabsTrigger>
					))}
				</TabsList>
				<TabsContent value={INFO_TAB}>
					<DoeInfoTab form={form} fileUrls={fileUrls} {...info} />
				</TabsContent>
				<TabsContent value={PLAN_TAB}>
					<DoePlanTab sections={form.sections} {...plan} />
				</TabsContent>
				{enabled.map(({ section, number }) => (
					<TabsContent key={section.id} value={section.id}>
						<DoeSectionTab
							section={section}
							number={number}
							fileUrls={fileUrls}
							actions={sectionActions}
						/>
					</TabsContent>
				))}
			</Tabs>
		</PageShell>
	)
}
