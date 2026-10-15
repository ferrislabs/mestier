import { AlertCircle } from 'lucide-react'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	maintenanceRecapPdfPath,
	useMaintenanceRecap,
	usePlanMaintenanceLines,
} from '#/hooks/use-maintenance-recap'
import { mutationErrorMessage } from '#/lib/api-error'
import { downloadAuthenticatedFile } from '#/lib/download-file'
import {
	planErrorMessage,
	recapLoadErrorMessage,
} from '#/pages/maintenance/lib/recap-errors'
import { lineIdsOf, pruneSelection } from '#/pages/maintenance/lib/recap-lines'
import { shiftIsoWeek, weekLabel } from '#/pages/maintenance/lib/recap-week'
import { RecapUI } from '#/pages/maintenance/ui/recap-ui'

interface RecapFeatureProps {
	/** ISO week, `2026-W43`. The route keeps it in the address. */
	week: string
	onWeekChange: (week: string) => void
}

/** Mounts the weekly recap. Router-agnostic: the week comes in as a prop. */
export function RecapFeature({ week, onWeekChange }: RecapFeatureProps) {
	const { activeOrganization } = useActiveOrganization()

	if (!activeOrganization) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Organisation indisponible</p>
					<p className="text-sm text-muted-foreground">
						Le récap d’entretien nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<RecapScreen
			key={activeOrganization.id}
			organizationId={activeOrganization.id}
			organizationName={activeOrganization.name}
			week={week}
			onWeekChange={onWeekChange}
		/>
	)
}

function RecapScreen({
	organizationId,
	organizationName,
	week,
	onWeekChange,
}: RecapFeatureProps & { organizationId: string; organizationName: string }) {
	const recap = useMaintenanceRecap(organizationId, week)
	const plan = usePlanMaintenanceLines()

	const groups = recap.data?.data.groups ?? []
	const [ticked, setTicked] = useState<ReadonlySet<string>>(new Set())
	const [notice, setNotice] = useState<string | null>(null)
	const [isDownloadingPdf, setIsDownloadingPdf] = useState(false)
	const [pdfError, setPdfError] = useState<string | null>(null)

	// Derived, not synced by an effect: a line that left the recap (planned
	// here or by someone else) simply stops counting.
	const selected = pruneSelection(ticked, groups)

	function changeWeek(next: string) {
		setTicked(new Set())
		setNotice(null)
		setPdfError(null)
		plan.reset()
		onWeekChange(next)
	}

	function toggleLine(lineId: string) {
		const next = new Set(selected)
		if (!next.delete(lineId)) next.add(lineId)
		setTicked(next)
	}

	function toggleAll() {
		const everyLine = lineIdsOf(groups)
		setTicked(
			selected.size === everyLine.length ? new Set() : new Set(everyLine),
		)
	}

	function planSelected() {
		// Sent in the order of the recap, not in the order of the clicks.
		const lineIds = lineIdsOf(groups).filter((id) => selected.has(id))
		setNotice(null)
		plan
			.mutateAsync({
				path: { organization_id: organizationId },
				body: { line_ids: lineIds, week },
			} as never)
			.then(() => {
				setTicked(new Set())
				setNotice(
					lineIds.length === 1
						? '1 prestation planifiée : sa tâche attend, sans date, dans le tableau.'
						: `${lineIds.length} prestations planifiées : leurs tâches attendent, sans date, dans le tableau.`,
				)
			})
			.catch(() => {
				// Shown through `plan.error`; this only keeps the rejection handled.
			})
	}

	async function downloadPdf() {
		setPdfError(null)
		setIsDownloadingPdf(true)
		try {
			await downloadAuthenticatedFile(
				maintenanceRecapPdfPath(organizationId, week),
				`recap-entretien-${week}.pdf`,
			)
		} catch (error) {
			setPdfError(mutationErrorMessage(error))
		} finally {
			setIsDownloadingPdf(false)
		}
	}

	return (
		<RecapUI
			organizationName={organizationName}
			weekLabel={weekLabel(week)}
			groups={groups}
			selected={selected}
			isLoading={recap.isLoading}
			loadError={recapLoadErrorMessage(recap.error)}
			onRetry={() => void recap.refetch()}
			planError={planErrorMessage(plan.error)}
			notice={notice}
			pdfError={pdfError}
			isPlanning={plan.isPending}
			isDownloadingPdf={isDownloadingPdf}
			onPreviousWeek={() => changeWeek(shiftIsoWeek(week, -1))}
			onNextWeek={() => changeWeek(shiftIsoWeek(week, 1))}
			onToggleLine={toggleLine}
			onToggleAll={toggleAll}
			onPlan={planSelected}
			onDownloadPdf={() => void downloadPdf()}
		/>
	)
}
