import { Check, Loader2, Lock, Minus, Plus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { cn } from '#/lib/utils'
import {
	type OverwriteBitName,
	readBitState,
	type TriState,
} from '#/pages/chat/lib/permission-bits'
import {
	EVERYONE_KEY,
	type PermissionTarget,
} from '#/pages/chat/lib/permission-targets'

export interface ChannelPermissionsSectionUIProps {
	isLoading: boolean
	isPrivate: boolean
	onTogglePrivate: (next: boolean) => void
	targets: PermissionTarget[]
	selectedKey: string
	onSelectKey: (key: string) => void
	addableRoles: { id: string; name: string }[]
	onAddRole: (roleId: string) => void
	onChangeBit: (key: string, bit: OverwriteBitName, state: TriState) => void
	onRemoveTarget: (key: string) => void
}

interface BitDefinition {
	bit: OverwriteBitName
	title: string
	description: string
}

const BITS: readonly BitDefinition[] = [
	{
		bit: 'VIEW_CHANNEL',
		title: 'Voir le canal',
		description:
			'Permet de voir ce canal. En le refusant à @everyone, le canal devient privé.',
	},
	{
		bit: 'SEND_MESSAGES',
		title: 'Envoyer des messages',
		description: 'Permet d’envoyer des messages dans ce canal.',
	},
]

const STATES: readonly {
	state: TriState
	label: string
	icon: typeof Check
	active: string
}[] = [
	{
		state: 'deny',
		label: 'Refuser',
		icon: X,
		active: 'bg-destructive/15 text-destructive',
	},
	{
		state: 'inherit',
		label: 'Hériter',
		icon: Minus,
		active: 'bg-accent text-foreground',
	},
	{
		state: 'allow',
		label: 'Autoriser',
		icon: Check,
		active: 'bg-primary/15 text-primary',
	},
]

export function ChannelPermissionsSectionUI({
	isLoading,
	isPrivate,
	onTogglePrivate,
	targets,
	selectedKey,
	onSelectKey,
	addableRoles,
	onAddRole,
	onChangeBit,
	onRemoveTarget,
}: ChannelPermissionsSectionUIProps) {
	const selected =
		targets.find((target) => target.key === selectedKey) ?? targets[0]

	return (
		<>
			<div className="flex flex-col gap-1">
				<h2 className="font-semibold text-lg">Permissions du canal</h2>
				<p className="text-muted-foreground text-sm">
					Personnalise qui peut faire quoi dans ce canal.
				</p>
			</div>

			<div className="flex items-start justify-between gap-4 rounded-lg border bg-card p-4">
				<div className="flex items-start gap-3">
					<Lock className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
					<div>
						<p className="font-medium text-sm">Canal privé</p>
						<p className="text-muted-foreground text-xs">
							En définissant un canal comme « privé », seuls les rôles et
							membres sélectionnés pourront le voir.
						</p>
					</div>
				</div>
				<Switch
					aria-label="Canal privé"
					checked={isPrivate}
					onCheckedChange={onTogglePrivate}
				/>
			</div>

			<div className="flex flex-col gap-4 border-t pt-6">
				<h3 className="font-semibold text-base">Permissions avancées</h3>
				{isLoading ? (
					<Loader2 className="size-4 animate-spin text-muted-foreground" />
				) : null}
				<div className="grid gap-6 sm:grid-cols-[12rem_minmax(0,1fr)]">
					<div className="flex flex-col gap-2">
						<div className="flex items-center justify-between text-muted-foreground text-xs">
							<span>Rôles</span>
						</div>
						<ul className="flex flex-col gap-0.5">
							{targets.map((target) => (
								<li key={target.key}>
									<button
										type="button"
										aria-pressed={target.key === selected?.key}
										onClick={() => onSelectKey(target.key)}
										className={cn(
											'w-full truncate rounded-md px-2.5 py-1.5 text-left text-sm hover:bg-accent',
											target.key === selected?.key && 'bg-accent font-medium',
										)}
									>
										{target.name}
									</button>
								</li>
							))}
						</ul>
						{addableRoles.length > 0 ? (
							<Select value="" onValueChange={onAddRole}>
								<SelectTrigger
									aria-label="Ajouter un rôle"
									className="h-8 w-full text-xs"
								>
									<Plus className="size-3.5" />
									<SelectValue placeholder="Ajouter un rôle" />
								</SelectTrigger>
								<SelectContent>
									{addableRoles.map((role) => (
										<SelectItem key={role.id} value={role.id}>
											{role.name}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						) : null}
					</div>

					{selected ? (
						<div className="flex min-w-0 flex-col gap-1">
							<h4 className="font-semibold text-sm">
								Permissions de canal · {selected.name}
							</h4>
							<ul className="flex flex-col divide-y">
								{BITS.map(({ bit, title, description }) => {
									const current = readBitState(
										selected.allow,
										selected.deny,
										bit,
									)
									return (
										<li
											key={bit}
											className="flex items-start justify-between gap-4 py-4"
										>
											<div className="min-w-0">
												<p className="font-medium text-sm">{title}</p>
												<p className="text-muted-foreground text-xs">
													{description}
												</p>
											</div>
											<div
												role="group"
												aria-label={title}
												className="flex shrink-0 overflow-hidden rounded-md border"
											>
												{STATES.map(({ state, label, icon: Icon, active }) => (
													<button
														key={state}
														type="button"
														aria-label={`${label} : ${title}`}
														aria-pressed={current === state}
														onClick={() =>
															onChangeBit(selected.key, bit, state)
														}
														className={cn(
															'flex size-8 items-center justify-center text-muted-foreground hover:bg-accent',
															current === state && active,
														)}
													>
														<Icon className="size-4" />
													</button>
												))}
											</div>
										</li>
									)
								})}
							</ul>
							{selected.key !== EVERYONE_KEY ? (
								<Button
									type="button"
									variant="outline"
									size="sm"
									className="self-start"
									onClick={() => onRemoveTarget(selected.key)}
								>
									Retirer {selected.name}
								</Button>
							) : null}
						</div>
					) : null}
				</div>
			</div>
		</>
	)
}
