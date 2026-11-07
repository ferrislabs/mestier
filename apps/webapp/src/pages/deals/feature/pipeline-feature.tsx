import { AlertCircle } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { CURRENT_PERSON_ID } from '#/pages/customers/feature/mocks/people'
import { todayIso } from '#/pages/customers/lib/format'
import { useDealQuotes } from '#/pages/deals/feature/mocks/deal-quotes'
import { useDeals } from '#/pages/deals/feature/use-deals'
import type { DealDraft, DealStage } from '#/pages/deals/lib/deal-types'
import { findDeal, pipelineSummary } from '#/pages/deals/lib/pipeline'
import {
	DEFAULT_FILTERS,
	type DealFilters,
	filterDeals,
	hasActiveFilters,
} from '#/pages/deals/lib/pipeline-filters'
import { PipelineUI } from '#/pages/deals/ui/pipeline-ui'

const NOTICE_DURATION_MS = 6000

interface PipelineFeatureProps {
	filters: DealFilters
	onFiltersChange: (filters: DealFilters) => void
}

export function PipelineFeature(props: PipelineFeatureProps) {
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
						Le pipeline nécessite une organisation active.
					</p>
				</div>
			</div>
		)
	}

	return (
		<Pipeline
			key={activeOrganization.id}
			organizationSlug={activeOrganization.slug}
			{...props}
		/>
	)
}

function Pipeline({
	organizationSlug,
	filters,
	onFiltersChange,
}: PipelineFeatureProps & { organizationSlug: string }) {
	const store = useDeals()
	const quotes = useDealQuotes()
	const canManage = useHasPermission('MANAGE_CUSTOMERS')
	const today = todayIso()
	const [query, setQuery] = useState(filters.query)
	const [creating, setCreating] = useState(false)
	const [selectedId, setSelectedId] = useState<string | null>(null)
	const [losingId, setLosingId] = useState<string | null>(null)
	const [notice, setNotice] = useState<string | null>(null)

	useEffect(() => {
		if (!notice) return
		const timer = setTimeout(() => setNotice(null), NOTICE_DURATION_MS)
		return () => clearTimeout(timer)
	}, [notice])

	const active: DealFilters = { ...filters, query }
	const visible = filterDeals(store.deals, store.customers, active)

	function change(patch: Partial<DealFilters>) {
		if (patch.query !== undefined) setQuery(patch.query)
		onFiltersChange({ ...active, ...patch })
	}

	function reset() {
		setQuery(DEFAULT_FILTERS.query)
		onFiltersChange(DEFAULT_FILTERS)
	}

	function moveTo(dealId: string, stage: DealStage) {
		const deal = findDeal(store.deals, dealId)
		if (!deal || deal.stage === stage) return
		if (stage === 'lost') {
			setLosingId(dealId)
			return
		}
		setNotice(store.move(dealId, stage))
	}

	function create(draft: DealDraft) {
		store.create(draft)
	}

	return (
		<PipelineUI
			deals={visible}
			customers={store.customers}
			quotes={quotes}
			summary={pipelineSummary(store.deals, store.conversions, today)}
			filters={active}
			canReset={hasActiveFilters(active)}
			today={today}
			organizationSlug={organizationSlug}
			defaultOwnerId={CURRENT_PERSON_ID}
			canManage={canManage}
			isLoading={store.isLoading}
			error={store.error}
			notice={notice}
			creating={creating}
			selected={findDeal(store.deals, selectedId)}
			losing={findDeal(store.deals, losingId)}
			onRetry={store.retry}
			onFiltersChange={change}
			onReset={reset}
			onDismissNotice={() => setNotice(null)}
			onCreatingChange={setCreating}
			onCreate={create}
			onOpen={setSelectedId}
			onCloseSelected={() => setSelectedId(null)}
			onMove={moveTo}
			onSave={store.update}
			onDelete={(dealId) => store.remove(dealId, quotes)}
			onCancelLoss={() => setLosingId(null)}
			onConfirmLoss={(dealId, reason) => {
				store.move(dealId, 'lost', reason)
				setNotice(null)
				setLosingId(null)
			}}
		/>
	)
}
