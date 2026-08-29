import { ChevronRight, Loader2, Plus, Trash2 } from 'lucide-react'
import type * as React from 'react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { Project } from '#/hooks/use-projects'
import type { SupplierInvoiceLine } from '#/hooks/use-supplier-invoices'
import { cn } from '#/lib/utils'
import { centsToEuros, eurosToCents } from '#/pages/invoices/types'
import { formatMoney } from '#/pages/purchase/types'

interface AllocationRow {
	clientId: string
	projectId: string
	amountEuros: string
}

interface SupplierInvoiceLineAllocationEditorProps {
	line: SupplierInvoiceLine
	projects: Project[]
	isOpen: boolean
	isSaving?: boolean
	onOpenChange: (open: boolean) => void
	onSave: (shares: { projectId: string; amountCents: number }[]) => void
}

let clientIdSeq = 0
function nextClientId() {
	clientIdSeq += 1
	return `alloc-${clientIdSeq}`
}

function rowsFromLine(line: SupplierInvoiceLine): AllocationRow[] {
	return line.allocations.map((allocation) => ({
		clientId: nextClientId(),
		projectId: allocation.project_id,
		amountEuros: centsToEuros(allocation.amount_cents),
	}))
}

/**
 * One line, one editor. Not a form submitted once: `PUT
 * .../allocations` is a full-replace, so "save" here always sends the
 * complete row list, never a delta — same contract `SupplierInvoiceService
 * ::replace_line_allocations` enforces server-side.
 *
 * The unallocated remainder is always visible (#340's own warning: a line
 * silently 80 % allocated is a cost that quietly went missing), computed
 * from the same rows the user is editing, not from what was last saved.
 */
export function SupplierInvoiceLineAllocationEditor({
	line,
	projects,
	isOpen,
	isSaving,
	onOpenChange,
	onSave,
}: SupplierInvoiceLineAllocationEditorProps) {
	const [rows, setRows] = useState<AllocationRow[]>(() => rowsFromLine(line))

	const allocatedCents = rows.reduce(
		(sum, row) => sum + eurosToCents(row.amountEuros || '0'),
		0,
	)
	const unallocatedCents = line.line_total_cents - allocatedCents
	const isFullyAllocated = unallocatedCents === 0 && rows.length > 0

	const patchRow = (clientId: string, patch: Partial<AllocationRow>) => {
		setRows((current) =>
			current.map((row) =>
				row.clientId === clientId ? { ...row, ...patch } : row,
			),
		)
	}

	const canSave = rows.every((row) => row.projectId && row.amountEuros.trim())

	return (
		<Collapsible
			open={isOpen}
			onOpenChange={onOpenChange}
			className="bg-card data-[state=open]:bg-muted/20"
		>
			<CollapsibleTrigger className="flex w-full items-center gap-3 px-4 py-3 text-left">
				<ChevronRight
					className={cn(
						'size-4 shrink-0 text-muted-foreground transition-transform',
						isOpen && 'rotate-90',
					)}
				/>
				<span className="min-w-0 flex-1">
					<span className="block truncate text-sm font-medium">
						{line.label}
					</span>
					<span className="block truncate text-xs text-muted-foreground">
						{isFullyAllocated
							? 'Entièrement attribuée'
							: unallocatedCents === line.line_total_cents
								? 'Non attribuée'
								: `${formatMoney(unallocatedCents)} non attribué`}
					</span>
				</span>
				<span className="shrink-0 text-sm font-semibold tabular-nums">
					{formatMoney(line.line_total_cents)}
				</span>
			</CollapsibleTrigger>

			<CollapsibleContent>
				<div className="space-y-3 border-t px-4 py-4">
					{rows.map((row) => (
						<div
							key={row.clientId}
							className="grid items-end gap-3 sm:grid-cols-[1fr_160px_auto]"
						>
							<Field label="Chantier">
								<Select
									value={row.projectId}
									onValueChange={(projectId) =>
										patchRow(row.clientId, { projectId })
									}
								>
									<SelectTrigger className="w-full">
										<SelectValue placeholder="Choisir un chantier" />
									</SelectTrigger>
									<SelectContent>
										{projects.map((project) => (
											<SelectItem key={project.id} value={project.id}>
												{project.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</Field>
							<Field label="Montant">
								<Input
									inputMode="decimal"
									value={row.amountEuros}
									onChange={(event) =>
										patchRow(row.clientId, { amountEuros: event.target.value })
									}
									placeholder="0,00"
								/>
							</Field>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								onClick={() =>
									setRows((current) =>
										current.filter((r) => r.clientId !== row.clientId),
									)
								}
							>
								<Trash2 />
								<span className="sr-only">Retirer cette attribution</span>
							</Button>
						</div>
					))}

					<div className="flex items-center justify-between gap-3">
						<Button
							type="button"
							variant="outline"
							size="sm"
							onClick={() =>
								setRows((current) => [
									...current,
									{
										clientId: nextClientId(),
										projectId: '',
										amountEuros:
											unallocatedCents > 0
												? centsToEuros(unallocatedCents)
												: '',
									},
								])
							}
						>
							<Plus />
							Ajouter un chantier
						</Button>

						<Button
							type="button"
							size="sm"
							disabled={!canSave || isSaving}
							onClick={() =>
								onSave(
									rows.map((row) => ({
										projectId: row.projectId,
										amountCents: eurosToCents(row.amountEuros),
									})),
								)
							}
						>
							{isSaving ? <Loader2 className="animate-spin" /> : null}
							Enregistrer
						</Button>
					</div>
				</div>
			</CollapsibleContent>
		</Collapsible>
	)
}

function Field({
	label,
	children,
}: {
	label: string
	children: React.ReactNode
}) {
	return (
		<div className="space-y-1.5">
			<Label className="text-xs text-muted-foreground">{label}</Label>
			{children}
		</div>
	)
}
