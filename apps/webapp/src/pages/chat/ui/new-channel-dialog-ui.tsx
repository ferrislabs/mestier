import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Checkbox } from '#/components/ui/checkbox'
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import { cn } from '#/lib/utils'
import {
	CHANNEL_KINDS,
	NEW_ITEM_KINDS,
	type NewItemKind,
} from '#/pages/chat/lib/channel-kinds'

export interface NewChannelDialogUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	name: string
	onChangeName: (value: string) => void
	/** Set when opened from a category: the channel goes in it, and only a
	 * channel can be created from there. */
	parentCategoryName?: string
	/** What is being made. A category is one of the choices, like a voice
	 * channel; it is not offered from inside a category. */
	channelKind: NewItemKind
	onChangeChannelKind: (kind: NewItemKind) => void
	isPrivate: boolean
	onChangePrivate: (isPrivate: boolean) => void
	roles: readonly { id: string; name: string }[]
	rolesLoading: boolean
	selectedRoleIds: ReadonlySet<string>
	onToggleRole: (roleId: string, selected: boolean) => void
	onSubmit: () => void
	isSubmitting: boolean
}

export function NewChannelDialogUI({
	open,
	onOpenChange,
	name,
	onChangeName,
	parentCategoryName,
	channelKind,
	onChangeChannelKind,
	isPrivate,
	onChangePrivate,
	roles,
	rolesLoading,
	selectedRoleIds,
	onToggleRole,
	onSubmit,
	isSubmitting,
}: NewChannelDialogUIProps) {
	const isCategory = channelKind === 'CATEGORY'

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

				<div className="flex flex-col gap-3 py-3">
					<div className="flex flex-col gap-1.5">
						<Label htmlFor="new-channel-name">
							{isCategory ? 'Nom de la catégorie' : 'Nom du canal'}
						</Label>
						<Input
							id="new-channel-name"
							value={name}
							onChange={(event) => onChangeName(event.target.value)}
							placeholder={isCategory ? 'Chantiers' : 'général'}
						/>
					</div>

					<fieldset className="flex flex-col gap-1.5">
						<legend className="mb-1.5 font-medium text-sm">Type</legend>
						<div role="radiogroup" className="grid gap-2">
							{(parentCategoryName ? CHANNEL_KINDS : NEW_ITEM_KINDS).map(
								(option) => {
									const Icon = option.icon
									const selected = channelKind === option.value
									return (
										<label
											key={option.value}
											className={cn(
												'flex cursor-pointer items-start gap-3 rounded-lg border border-border p-3 transition-colors has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring hover:bg-accent/50',
												selected && 'border-primary bg-accent',
											)}
										>
											<input
												type="radio"
												name="new-channel-kind"
												value={option.value}
												checked={selected}
												onChange={() => onChangeChannelKind(option.value)}
												className="sr-only"
											/>
											<span
												className={cn(
													'flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground',
													selected && 'bg-primary text-primary-foreground',
												)}
											>
												<Icon aria-hidden className="size-4" />
											</span>
											<span className="min-w-0">
												<span className="block font-medium text-sm">
													{option.title}
												</span>
												<span className="block text-muted-foreground text-xs">
													{option.description}
												</span>
											</span>
										</label>
									)
								},
							)}
						</div>
					</fieldset>

					<>
						<div className="flex items-start justify-between gap-4 rounded-lg border border-border p-3">
							<div className="min-w-0">
								<Label htmlFor="new-channel-private" className="font-medium">
									Salon privé
								</Label>
								<p className="text-muted-foreground text-xs">
									{isCategory
										? 'Seuls les rôles choisis peuvent voir les canaux de cette catégorie.'
										: 'Seuls les rôles choisis peuvent voir et écrire dans ce canal.'}
								</p>
							</div>
							<Switch
								id="new-channel-private"
								checked={isPrivate}
								onCheckedChange={onChangePrivate}
							/>
						</div>

						{isPrivate ? (
							<fieldset className="flex flex-col gap-1.5">
								<legend className="mb-1.5 font-medium text-sm">
									Rôles autorisés
								</legend>
								{rolesLoading ? (
									<Loader2 className="size-4 animate-spin text-muted-foreground" />
								) : (
									<ul className="flex max-h-40 flex-col gap-1 overflow-y-auto">
										{roles.map((role) => (
											<li key={role.id}>
												<label className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent/50">
													<Checkbox
														checked={selectedRoleIds.has(role.id)}
														onCheckedChange={(checked) =>
															onToggleRole(role.id, checked === true)
														}
													/>
													{role.name}
												</label>
											</li>
										))}
									</ul>
								)}
							</fieldset>
						) : null}
					</>
				</div>

				<DialogFooter>
					<Button
						type="button"
						onClick={onSubmit}
						disabled={
							isSubmitting ||
							name.trim().length === 0 ||
							(isPrivate && selectedRoleIds.size === 0)
						}
					>
						{isSubmitting ? <Loader2 className="size-4 animate-spin" /> : null}
						Créer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
