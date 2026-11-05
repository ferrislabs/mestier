import { ChevronRight, Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { setAbsenceTypeArchived, typeUsage } from '#/pages/hr/lib/rh/absences'
import { countModeOption } from '#/pages/hr/lib/rh/absences-view'
import type { AbsenceType, RhState } from '#/pages/hr/lib/rh/types'
import { AbsenceTypeDialog } from '#/pages/hr/ui/rh/absences/absence-type-dialog'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'
import { TypeDot } from '#/pages/hr/ui/rh/shared/type-chip'

type EditorState = { type: AbsenceType | null; open: boolean }

export function AbsenceTypesTab() {
	const updateRh = useRhUpdate()
	const state = useRhState()
	const [editor, setEditor] = useState<EditorState>({ type: null, open: false })
	const active = state.absenceTypes.filter((type) => !type.archived)
	const archived = state.absenceTypes.filter((type) => type.archived)

	function archive(type: AbsenceType, value: boolean) {
		updateRh((current) => setAbsenceTypeArchived(current, type.id, value))
	}

	return (
		<section aria-labelledby="types-title">
			<div className="mb-3 flex flex-wrap items-end justify-between gap-3">
				<div>
					<h2 id="types-title" className="font-semibold text-base">
						Motifs d’absence
					</h2>
					<p className="mt-0.5 text-muted-foreground text-sm">
						Chaque motif fixe le décompte des jours, le compte débité et le
						circuit de validation.
					</p>
				</div>
				<Button
					type="button"
					size="sm"
					onClick={() => setEditor({ type: null, open: true })}
				>
					<Plus />
					Nouveau motif
				</Button>
			</div>
			{active.length === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">Aucun motif actif.</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						Crée un motif pour pouvoir poser des absences.
					</p>
				</div>
			) : (
				<TypesTable
					state={state}
					types={active}
					onEdit={(type) => setEditor({ type, open: true })}
					onToggle={archive}
				/>
			)}
			{archived.length > 0 && (
				<details className="group mt-6">
					<summary className="flex w-fit cursor-pointer select-none list-none items-center gap-1.5 rounded-md py-1 pr-2 font-medium text-muted-foreground text-sm outline-none hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 [&::-webkit-details-marker]:hidden">
						<ChevronRight className="size-4 transition-transform group-open:rotate-90" />
						Motifs archivés
						<span className="rounded-full bg-muted px-1.5 text-[11px] tabular leading-4">
							{archived.length}
						</span>
					</summary>
					<div className="mt-3">
						<TypesTable
							state={state}
							types={archived}
							onEdit={(type) => setEditor({ type, open: true })}
							onToggle={archive}
						/>
					</div>
				</details>
			)}
			<AbsenceTypeDialog
				open={editor.open}
				type={editor.type}
				onOpenChange={(open) => setEditor((current) => ({ ...current, open }))}
			/>
		</section>
	)
}

function TypesTable({
	state,
	types,
	onEdit,
	onToggle,
}: {
	state: RhState
	types: readonly AbsenceType[]
	onEdit: (type: AbsenceType) => void
	onToggle: (type: AbsenceType, archived: boolean) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Motif</TableHead>
						<TableHead>Décompte</TableHead>
						<TableHead>Compte débité</TableHead>
						<TableHead>Rémunéré</TableHead>
						<TableHead>Validation</TableHead>
						<TableHead>Justificatif</TableHead>
						<TableHead className="text-right">Utilisations</TableHead>
						<TableHead className="pr-4 text-right">
							<span className="sr-only">Actions</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{types.map((type) => {
						const account = state.accounts.find(
							(other) => other.id === type.accountId,
						)
						return (
							<TableRow key={type.id}>
								<TableCell className="py-2.5 pl-4">
									<span className="flex items-center gap-2 font-medium">
										<TypeDot hue={type.hue} className="size-2.5" />
										{type.name}
									</span>
								</TableCell>
								<TableCell>{countModeOption(type.countMode).label}</TableCell>
								<TableCell
									className={account ? undefined : 'text-muted-foreground'}
								>
									{account?.name ?? 'Aucun'}
								</TableCell>
								<TableCell>{type.paid ? 'Oui' : 'Non'}</TableCell>
								<TableCell>
									{type.needsApproval ? 'Requise' : 'Automatique'}
								</TableCell>
								<TableCell
									className={
										type.needsProof ? undefined : 'text-muted-foreground'
									}
								>
									{type.needsProof ? 'Attendu' : 'Non'}
								</TableCell>
								<TableCell className="tabular text-right">
									{typeUsage(state, type.id)}
								</TableCell>
								<TableCell className="pr-3 text-right">
									<div className="flex justify-end gap-1">
										<Button
											type="button"
											variant="ghost"
											size="sm"
											aria-label={`Modifier le motif ${type.name}`}
											onClick={() => onEdit(type)}
										>
											Modifier
										</Button>
										<Button
											type="button"
											variant="ghost"
											size="sm"
											className="text-muted-foreground"
											aria-label={`${type.archived ? 'Réactiver' : 'Archiver'} le motif ${type.name}`}
											onClick={() => onToggle(type, !type.archived)}
										>
											{type.archived ? 'Réactiver' : 'Archiver'}
										</Button>
									</div>
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
