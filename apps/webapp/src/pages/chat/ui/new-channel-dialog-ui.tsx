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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '#/components/ui/tabs'
import type { Category } from '#/hooks/use-chat'

const NO_CATEGORY_VALUE = '__none__'

export interface NewChannelDialogUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	kind: 'channel' | 'category'
	onChangeKind: (kind: 'channel' | 'category') => void
	name: string
	onChangeName: (value: string) => void
	categories: Category[]
	categoryId: string | null
	onChangeCategoryId: (value: string | null) => void
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
	categories,
	categoryId,
	onChangeCategoryId,
	onSubmit,
	isSubmitting,
}: NewChannelDialogUIProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Créer</DialogTitle>
				</DialogHeader>

				<Tabs
					value={kind}
					onValueChange={(value) =>
						onChangeKind(value as 'channel' | 'category')
					}
				>
					<TabsList>
						<TabsTrigger value="channel">Canal</TabsTrigger>
						<TabsTrigger value="category">Catégorie</TabsTrigger>
					</TabsList>

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
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="new-channel-category">Catégorie</Label>
							<Select
								value={categoryId ?? NO_CATEGORY_VALUE}
								onValueChange={(value) =>
									onChangeCategoryId(value === NO_CATEGORY_VALUE ? null : value)
								}
							>
								<SelectTrigger id="new-channel-category">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={NO_CATEGORY_VALUE}>
										Sans catégorie
									</SelectItem>
									{categories.map((category) => (
										<SelectItem key={category.id} value={category.id}>
											{category.name}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
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
