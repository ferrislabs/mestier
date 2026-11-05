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
import {
	formatRate,
	setSurchargeArchived,
	surchargeUsage,
} from '#/pages/hr/lib/rh/extra'
import { kindLabel } from '#/pages/hr/lib/rh/extra-view'
import type { RhState, Surcharge } from '#/pages/hr/lib/rh/types'
import { SurchargeDialog } from '#/pages/hr/ui/rh/extra/surcharge-dialog'
import { useRhState, useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

type EditorState = { surcharge: Surcharge | null; open: boolean }

export function SurchargesTab() {
	const updateRh = useRhUpdate()
	const state = useRhState()
	const [editor, setEditor] = useState<EditorState>({
		surcharge: null,
		open: false,
	})
	const active = state.surcharges.filter((surcharge) => !surcharge.archived)
	const archived = state.surcharges.filter((surcharge) => surcharge.archived)

	function archive(surcharge: Surcharge, value: boolean) {
		updateRh((current) => setSurchargeArchived(current, surcharge.id, value))
	}

	return (
		<section aria-labelledby="surcharges-title">
			<div className="mb-3 flex flex-wrap items-end justify-between gap-3">
				<div>
					<h2 id="surcharges-title" className="font-semibold text-base">
						Barème des majorations
					</h2>
					<p className="mt-0.5 max-w-2xl text-muted-foreground text-sm">
						Les déclarations déjà saisies gardent leur taux d’origine. Une
						majoration archivée n’est plus proposée dans le formulaire.
					</p>
				</div>
				<Button
					type="button"
					size="sm"
					onClick={() => setEditor({ surcharge: null, open: true })}
				>
					<Plus />
					Nouvelle majoration
				</Button>
			</div>
			{active.length === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">Aucune majoration active.</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						Les heures extra se déclarent alors avec un taux libre ou sans
						majoration.
					</p>
				</div>
			) : (
				<SurchargesTable
					state={state}
					surcharges={active}
					onEdit={(surcharge) => setEditor({ surcharge, open: true })}
					onToggle={archive}
				/>
			)}
			{archived.length > 0 && (
				<details className="group mt-6">
					<summary className="flex w-fit cursor-pointer select-none list-none items-center gap-1.5 rounded-md py-1 pr-2 font-medium text-muted-foreground text-sm outline-none hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 [&::-webkit-details-marker]:hidden">
						<ChevronRight className="size-4 transition-transform group-open:rotate-90" />
						Majorations archivées
						<span className="tabular rounded-full bg-muted px-1.5 text-[11px] leading-4">
							{archived.length}
						</span>
					</summary>
					<div className="mt-3">
						<SurchargesTable
							state={state}
							surcharges={archived}
							onEdit={(surcharge) => setEditor({ surcharge, open: true })}
							onToggle={archive}
						/>
					</div>
				</details>
			)}
			<SurchargeDialog
				open={editor.open}
				surcharge={editor.surcharge}
				onOpenChange={(open) => setEditor((current) => ({ ...current, open }))}
			/>
		</section>
	)
}

function SurchargesTable({
	state,
	surcharges,
	onEdit,
	onToggle,
}: {
	state: RhState
	surcharges: readonly Surcharge[]
	onEdit: (surcharge: Surcharge) => void
	onToggle: (surcharge: Surcharge, archived: boolean) => void
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Nom</TableHead>
						<TableHead>Taux</TableHead>
						<TableHead>Type</TableHead>
						<TableHead className="text-right">Utilisations</TableHead>
						<TableHead className="pr-4 text-right">
							<span className="sr-only">Actions</span>
						</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{surcharges.map((surcharge) => (
						<TableRow key={surcharge.id}>
							<TableCell className="py-2.5 pl-4 font-medium">
								{surcharge.name}
							</TableCell>
							<TableCell className="tabular">
								{formatRate(surcharge.rate)}
							</TableCell>
							<TableCell>{kindLabel(surcharge.kind)}</TableCell>
							<TableCell className="tabular text-right">
								{surchargeUsage(state, surcharge.id)}
							</TableCell>
							<TableCell className="pr-3 text-right">
								<div className="flex justify-end gap-1">
									<Button
										type="button"
										variant="ghost"
										size="sm"
										aria-label={`Modifier la majoration ${surcharge.name}`}
										onClick={() => onEdit(surcharge)}
									>
										Modifier
									</Button>
									<Button
										type="button"
										variant="ghost"
										size="sm"
										className="text-muted-foreground"
										aria-label={`${surcharge.archived ? 'Réactiver' : 'Archiver'} la majoration ${surcharge.name}`}
										onClick={() => onToggle(surcharge, !surcharge.archived)}
									>
										{surcharge.archived ? 'Réactiver' : 'Archiver'}
									</Button>
								</div>
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
		</div>
	)
}
