import { Link } from '@tanstack/react-router'
import { AlertCircle, Search, Users } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	StatusBadge,
} from '#/components/ui/surface'
import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import {
	customerDisplayName,
	customerPipelineStageLabel,
} from '#/pages/customers/types'
import {
	CUSTOMER_PIPELINE_STAGES,
	CustomerPipelineBoard,
} from '#/pages/customers/ui/customer-pipeline-board'

interface CustomerPipelineUIProps {
	customers: Customer[]
	organizationSlug: string
	error?: string | null
	isLoading?: boolean
	movingCustomerId?: string | null
	onMovePipelineStage: (
		customer: Customer,
		pipelineStage: CustomerPipelineStage,
	) => void
	onOpenCustomer?: (customer: Customer) => void
	onRetry?: () => void
}

export function CustomerPipelineUI({
	customers,
	organizationSlug,
	error,
	isLoading,
	movingCustomerId,
	onMovePipelineStage,
	onOpenCustomer,
	onRetry,
}: CustomerPipelineUIProps) {
	const [search, setSearch] = useState('')
	const [stageFilter, setStageFilter] = useState<'all' | CustomerPipelineStage>(
		'all',
	)
	const [draggedCustomerId, setDraggedCustomerId] = useState<string | null>(
		null,
	)

	const pipelineCustomers = useMemo(
		() => customers.filter((customer) => customer.status !== 'ARCHIVED'),
		[customers],
	)
	const filteredCustomers = useMemo(() => {
		const query = search.trim().toLowerCase()

		return pipelineCustomers.filter((customer) => {
			const matchesStage =
				stageFilter === 'all' || customer.pipeline_stage === stageFilter
			const matchesSearch =
				!query ||
				customerDisplayName(customer).toLowerCase().includes(query) ||
				customerPipelineStageLabel(customer.pipeline_stage)
					.toLowerCase()
					.includes(query) ||
				(customer.email ?? '').toLowerCase().includes(query) ||
				(customer.phone ?? '').toLowerCase().includes(query)

			return matchesStage && matchesSearch
		})
	}, [pipelineCustomers, search, stageFilter])
	const draggedCustomer =
		filteredCustomers.find((customer) => customer.id === draggedCustomerId) ??
		null
	const metrics = useMemo(() => {
		return {
			total: pipelineCustomers.length,
			active: pipelineCustomers.filter(
				(customer) =>
					customer.pipeline_stage !== 'WON' &&
					customer.pipeline_stage !== 'LOST',
			).length,
			quotes: pipelineCustomers.filter(
				(customer) => customer.pipeline_stage === 'QUOTE_SENT',
			).length,
			won: pipelineCustomers.filter(
				(customer) => customer.pipeline_stage === 'WON',
			).length,
		}
	}, [pipelineCustomers])

	return (
		<PageShell className="max-w-none overflow-x-hidden">
			<PageHeader
				title="Pipeline commercial"
				description="Pilotez les prospects par étape, de la qualification à la conversion."
				actions={
					<Button asChild variant="outline">
						<Link to={buildOrgPath(organizationSlug, '/crm/customers')}>
							<Users />
							Fichier clients
						</Link>
					</Button>
				}
			/>

			{error ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					{onRetry ? (
						<Button onClick={onRetry} variant="outline" size="sm">
							Réessayer
						</Button>
					) : null}
				</SectionCard>
			) : null}

			<div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
				<MetricCard
					label="Opportunités"
					value={metrics.total}
					hint="Prospects et clients actifs"
				/>
				<MetricCard
					label="En cours"
					value={metrics.active}
					hint="Hors gagné/perdu"
				/>
				<MetricCard
					label="Devis envoyés"
					value={metrics.quotes}
					hint="À suivre"
				/>
				<MetricCard
					label="Gagnés"
					value={metrics.won}
					hint="Convertis clients"
				/>
			</div>

			<SectionCard className="flex min-w-0 flex-col gap-4 overflow-hidden p-4">
				<div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
					<div className="min-w-0">
						<div className="flex flex-wrap items-center gap-2">
							<h2 className="font-semibold">Board prospects</h2>
							<StatusBadge tone="neutral">
								{filteredCustomers.length} carte
								{filteredCustomers.length > 1 ? 's' : ''}
							</StatusBadge>
						</div>
						<p className="mt-1 text-sm text-muted-foreground">
							Déplacez une carte vers Gagné pour convertir automatiquement le
							prospect en client.
						</p>
					</div>

					<div className="flex flex-col gap-2 sm:flex-row sm:items-center">
						<div className="relative">
							<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
							<Input
								value={search}
								onChange={(event) => setSearch(event.target.value)}
								placeholder="Rechercher…"
								className="pl-9 sm:w-64"
							/>
						</div>
						<Select
							value={stageFilter}
							onValueChange={(value) =>
								setStageFilter(value as 'all' | CustomerPipelineStage)
							}
						>
							<SelectTrigger className="w-full bg-card sm:w-48">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="all">Toutes les étapes</SelectItem>
								{CUSTOMER_PIPELINE_STAGES.map((stage) => (
									<SelectItem key={stage} value={stage}>
										{customerPipelineStageLabel(stage)}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>
				</div>

				<CustomerPipelineBoard
					customers={filteredCustomers}
					isLoading={isLoading}
					draggedCustomerId={draggedCustomerId}
					movingCustomerId={movingCustomerId}
					onDragStart={setDraggedCustomerId}
					onDragEnd={() => setDraggedCustomerId(null)}
					onMove={(customer, stage) => {
						if (customer.pipeline_stage === stage) return
						onMovePipelineStage(customer, stage)
					}}
					onDropOnStage={(stage) => {
						if (!draggedCustomer) return
						if (draggedCustomer.pipeline_stage === stage) return
						onMovePipelineStage(draggedCustomer, stage)
						setDraggedCustomerId(null)
					}}
					onOpenCustomer={onOpenCustomer}
				/>
			</SectionCard>
		</PageShell>
	)
}
