import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'

export interface NewChannelDialogUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	kind: 'channel' | 'category'
	onChangeKind: (kind: 'channel' | 'category') => void
	name: string
	onChangeName: (value: string) => void
	/** Set when opened from a category: the channel goes in it, and only a
	 * channel can be created from there. */
	parentCategoryName?: string
	onSubmit: () => void
	isSubmitting: boolean
}

export function NewChannelDialogUI({
	open,
	onOpenChange,
	kind,
	onChangeKind,
	name,
	onChangeName,
	parentCategoryName,
	onSubmit,
	isSubmitting,
}: NewChannelDialogUIProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>
						{parentCategoryName
							? `Nouveau canal dans ${parentCategoryName}`
							: 'Créer'}
					</DialogTitle>
				</DialogHeader>

				<Tabs
					value={kind}
					onValueChange={(value) =>
						onChangeKind(value as 'channel' | 'category')
					}
				>
					{parentCategoryName ? null : (
						<TabsList>
							<TabsTrigger value="channel">Canal</TabsTrigger>
							<TabsTrigger value="category">Catégorie</TabsTrigger>
						</TabsList>
					)}

					<TabsContent value="channel" className="flex flex-col gap-3 py-3">
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="new-channel-name">Nom du canal</Label>
							<Input
								id="new-channel-name"
								value={name}
								onChange={(event) => onChangeName(event.target.value)}
								placeholder="général"
							/>
						</div>
					</TabsContent>

					<TabsContent value="category" className="flex flex-col gap-3 py-3">
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="new-category-name">Nom de la catégorie</Label>
							<Input
								id="new-category-name"
								value={name}
								onChange={(event) => onChangeName(event.target.value)}
								placeholder="Général"
							/>
						</div>
					</TabsContent>
				</Tabs>

				<DialogFooter>
					<Button
						type="button"
						onClick={onSubmit}
						disabled={isSubmitting || name.trim().length === 0}
					>
						{isSubmitting ? <Loader2 className="size-4 animate-spin" /> : null}
						Créer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
