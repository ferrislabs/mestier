import {
	ChevronDown,
	Copy,
	Loader2,
	Webhook as WebhookIcon,
} from 'lucide-react'
import { DoubleClickConfirmButton } from '#/components/double-click-confirm-button'
import { Button } from '#/components/ui/button'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import type { Webhook } from '#/hooks/use-chat'

export interface ChannelWebhooksSectionUIProps {
	channelName: string | undefined
	webhooks: Webhook[]
	isLoading: boolean
	creatorName: (userId: string) => string | undefined
	newWebhookName: string
	onChangeNewWebhookName: (value: string) => void
	onCreate: () => void
	isCreating: boolean
	createdToken: string | null
	/** Where webhooks are called, up to and excluding the webhook id. */
	callUrlBase: string
	/** The full call URL of the webhook just created, token included. */
	createdCallUrl: string | null
	onDismissCreatedToken: () => void
	onRename: (id: string, name: string) => void
	onDelete: (id: string) => void
}

const HUES = [330, 140, 215, 35, 265, 175]

function hueOf(id: string) {
	let sum = 0
	for (const char of id) sum += char.charCodeAt(0)
	return HUES[sum % HUES.length]
}

function formatDate(value: string) {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) return ''
	return date.toLocaleDateString('fr-FR', {
		day: 'numeric',
		month: 'short',
		year: 'numeric',
	})
}

export function ChannelWebhooksSectionUI({
	channelName,
	webhooks,
	isLoading,
	creatorName,
	newWebhookName,
	onChangeNewWebhookName,
	onCreate,
	isCreating,
	createdToken,
	callUrlBase,
	createdCallUrl,
	onDismissCreatedToken,
	onRename,
	onDelete,
}: ChannelWebhooksSectionUIProps) {
	return (
		<>
			<div className="flex flex-col gap-2">
				<h2 className="font-semibold text-lg">
					Intégrations <span className="text-muted-foreground">›</span> Webhooks
				</h2>
				<p className="text-muted-foreground text-sm">
					Les webhooks permettent à d’autres applications de poster des messages
					dans ce canal. Le jeton n’est affiché qu’une seule fois, à la
					création.
				</p>
			</div>

			<div className="flex items-end gap-2 border-t pt-6">
				<div className="flex flex-1 flex-col gap-1.5">
					<Label htmlFor="new-webhook-name">Nom du webhook</Label>
					<Input
						id="new-webhook-name"
						placeholder="Ex. : Alertes de chantier"
						value={newWebhookName}
						onChange={(event) => onChangeNewWebhookName(event.target.value)}
					/>
				</div>
				<Button
					type="button"
					onClick={onCreate}
					disabled={isCreating || newWebhookName.trim().length === 0}
				>
					Créer
				</Button>
			</div>

			{createdToken ? (
				<div className="flex flex-col gap-2 rounded-md border border-warning bg-warning-soft p-3 text-xs">
					<p className="font-semibold text-warning">
						Copiez ce jeton maintenant — il ne sera plus jamais affiché.
					</p>
					{createdCallUrl ? (
						<div className="flex flex-col gap-1">
							<span className="text-muted-foreground">
								URL à appeler (POST)
							</span>
							<div className="flex items-center gap-2">
								<code className="flex-1 truncate rounded bg-background px-2 py-1">
									{createdCallUrl}
								</code>
								<button
									type="button"
									aria-label="Copier l’URL du webhook"
									onClick={() => navigator.clipboard?.writeText(createdCallUrl)}
								>
									<Copy className="size-3.5" />
								</button>
							</div>
						</div>
					) : null}
					<span className="text-muted-foreground">Jeton</span>
					<div className="flex items-center gap-2">
						<code className="flex-1 truncate rounded bg-background px-2 py-1">
							{createdToken}
						</code>
						<button
							type="button"
							aria-label="Copier le jeton"
							onClick={() => navigator.clipboard?.writeText(createdToken)}
						>
							<Copy className="size-3.5" />
						</button>
					</div>
					<button
						type="button"
						className="self-end underline"
						onClick={onDismissCreatedToken}
					>
						J’ai copié le jeton
					</button>
				</div>
			) : null}

			<div className="flex flex-col gap-3">
				<h3 className="font-semibold text-sm">
					Poste sur {channelName ? `#${channelName}` : 'ce canal'}
				</h3>
				{isLoading ? (
					<Loader2 className="size-4 animate-spin text-muted-foreground" />
				) : null}
				{!isLoading && webhooks.length === 0 ? (
					<p className="text-muted-foreground text-xs">
						Aucun webhook pour ce canal.
					</p>
				) : null}
				<ul className="flex flex-col gap-3">
					{webhooks.map((webhook) => {
						const creator = creatorName(webhook.created_by)
						return (
							<li key={webhook.id}>
								<Collapsible className="group/webhook rounded-lg border bg-card">
									<CollapsibleTrigger className="flex w-full items-center gap-3 p-3 text-left outline-none">
										<span
											className="flex size-10 shrink-0 items-center justify-center rounded-full text-white"
											style={{
												backgroundColor: `hsl(${hueOf(webhook.id)} 55% 45%)`,
											}}
										>
											<WebhookIcon className="size-5" aria-hidden />
										</span>
										<span className="min-w-0 flex-1">
											<span className="block truncate font-semibold text-sm">
												{webhook.name}
											</span>
											<span className="block truncate text-muted-foreground text-xs">
												Créé le {formatDate(webhook.created_at)}
												{creator ? ` par ${creator}` : ''}
											</span>
										</span>
										<ChevronDown className="size-4 shrink-0 text-muted-foreground transition-transform group-data-[state=open]/webhook:rotate-180" />
									</CollapsibleTrigger>
									<CollapsibleContent>
										<form
											className="flex flex-col gap-4 border-t p-3"
											onSubmit={(event) => {
												event.preventDefault()
												const data = new FormData(event.currentTarget)
												const name = String(data.get('name') ?? '').trim()
												if (name.length > 0 && name !== webhook.name) {
													onRename(webhook.id, name)
												}
											}}
										>
											<div className="flex flex-col gap-1.5">
												<Label htmlFor={`webhook-name-${webhook.id}`}>
													Nom
												</Label>
												<Input
													id={`webhook-name-${webhook.id}`}
													name="name"
													defaultValue={webhook.name}
												/>
											</div>
											<div className="flex flex-col gap-1.5">
												<Label htmlFor={`webhook-url-${webhook.id}`}>
													URL à appeler (POST)
												</Label>
												<Input
													id={`webhook-url-${webhook.id}`}
													readOnly
													className="font-mono text-xs"
													value={`${callUrlBase}/${webhook.id}/{jeton}`}
												/>
												<p className="text-muted-foreground text-xs">
													Remplace {'{jeton}'} par le jeton reçu à la création.
												</p>
											</div>
											<div className="flex gap-2">
												<Button type="submit" size="sm">
													Enregistrer
												</Button>
												<DoubleClickConfirmButton
													size="sm"
													variant="outline"
													className="text-destructive"
													label="Suppression de webhook"
													onConfirm={() => onDelete(webhook.id)}
												/>
											</div>
										</form>
									</CollapsibleContent>
								</Collapsible>
							</li>
						)
					})}
				</ul>
			</div>
		</>
	)
}
