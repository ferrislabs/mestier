import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'

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
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[85vh] max-w-xl flex-col gap-0 overflow-hidden">
				<DialogHeader className="border-b pb-4">
					<DialogTitle>Piocher dans le catalogue</DialogTitle>
					<DialogDescription>
						Cocher un produit en copie la fiche dans ce DOE. Décocher la retire.
					</DialogDescription>
				</DialogHeader>
				<div className="space-y-3 py-4">
					<Input
						value={search}
						placeholder="Rechercher un produit"
						aria-label="Rechercher un produit"
						onChange={(event) => onSearchChange(event.target.value)}
					/>
					{error ? <p className="text-sm text-destructive">{error}</p> : null}
				</div>
				<div className="flex-1 overflow-y-auto">
					{isLoading ? (
						<div className="flex items-center justify-center gap-2 p-8 text-sm text-muted-foreground">
							<Loader2 className="size-4 animate-spin" />
							Chargement du catalogue…
						</div>
					) : products.length === 0 ? (
						<p className="p-8 text-center text-sm text-muted-foreground">
							Aucun produit ne correspond.
						</p>
					) : (
						<ul className="divide-y">
							{products.map((product) => {
								const inputId = `doe-pick-${product.id}`
								return (
									<li key={product.id} className="flex items-center gap-3 py-2">
										<Checkbox
											id={inputId}
											checked={pickedIds.has(product.id)}
											disabled={isDrafting}
											onCheckedChange={(checked) =>
												onToggle(product.id, checked === true)
											}
										/>
										<label htmlFor={inputId} className="min-w-0 flex-1 text-sm">
											<span className="block truncate font-medium">
												{product.name}
											</span>
											{product.sku ? (
												<span className="text-xs text-muted-foreground">
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
				<DialogFooter className="border-t pt-4">
					<Button type="button" onClick={() => onOpenChange(false)}>
						Terminé
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
