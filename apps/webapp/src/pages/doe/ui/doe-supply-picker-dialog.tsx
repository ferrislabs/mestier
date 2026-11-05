import { Loader2 } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

export interface PickerProduct {
	id: string
	name: string
	sku?: string | null
}

interface DoeSupplyPickerDialogProps {
	open: boolean
	products: PickerProduct[]
	isLoading: boolean
	search: string
	pickedIds: ReadonlySet<string>
	isDrafting: boolean
	error: string | null
	onSearchChange: (search: string) => void
	onToggle: (productId: string, checked: boolean) => void
	onOpenChange: (open: boolean) => void
}

export function DoeSupplyPickerDialog({
	open,
	products,
	isLoading,
	search,
	pickedIds,
	isDrafting,
	error,
	onSearchChange,
	onToggle,
	onOpenChange,
}: DoeSupplyPickerDialogProps) {
	const baseId = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85dvh] flex-col gap-0 p-0 sm:max-w-xl">
				<DialogHeader className="border-b px-6 py-4 pr-12">
					<DialogTitle>Piocher dans le catalogue</DialogTitle>
					<DialogDescription>
						Cocher un produit en copie la fiche dans ce DOE. Décocher la retire.
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-2 px-6 py-4">
					<Label htmlFor={`${baseId}-search`} className="sr-only">
						Rechercher un produit
					</Label>
					<Input
						id={`${baseId}-search`}
						type="search"
						value={search}
						placeholder="Rechercher un produit"
						aria-label="Rechercher un produit"
						autoComplete="off"
						onChange={(event) => onSearchChange(event.target.value)}
					/>
					{error ? (
						<p role="alert" className="text-[13px] text-state-alert">
							{error}
						</p>
					) : null}
				</div>
				<div className="min-h-0 flex-1 overflow-y-auto border-t">
					{isLoading ? (
						<div className="flex items-center justify-center gap-2 px-6 py-10 text-muted-foreground text-sm">
							<Loader2 className="size-4 animate-spin" aria-hidden />
							Chargement du catalogue…
						</div>
					) : products.length === 0 ? (
						<p className="px-6 py-10 text-center text-muted-foreground text-sm">
							Aucun produit ne correspond.
						</p>
					) : (
						<ul className="divide-y">
							{products.map((product) => {
								const inputId = `${baseId}-${product.id}`
								return (
									<li
										key={product.id}
										className="flex items-start gap-3 px-6 py-3"
									>
										<input
											id={inputId}
											type="checkbox"
											checked={pickedIds.has(product.id)}
											disabled={isDrafting}
											onChange={(event) =>
												onToggle(product.id, event.target.checked)
											}
											className="mt-0.5 size-4 shrink-0 rounded-sm accent-primary outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed"
										/>
										<label
											htmlFor={inputId}
											className="grid min-w-0 flex-1 cursor-pointer gap-0.5"
										>
											<span className="font-medium text-sm">
												{product.name}
											</span>
											{product.sku ? (
												<span className="text-muted-foreground text-xs">
													{product.sku}
												</span>
											) : null}
										</label>
									</li>
								)
							})}
						</ul>
					)}
				</div>
				<DialogFooter className="border-t px-6 py-4">
					<Button type="button" onClick={() => onOpenChange(false)}>
						Terminé
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
