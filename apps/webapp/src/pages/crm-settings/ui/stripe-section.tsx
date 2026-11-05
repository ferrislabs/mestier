import { Link } from '@tanstack/react-router'
import { ArrowRight, Check, Copy, KeyRound } from 'lucide-react'
import { type FormEvent, useId, useRef, useState } from 'react'
import { FormField } from '#/components/form-field'
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
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { buildOrgPath } from '#/modules/org-path'
import {
	SECRET_MASK,
	STRIPE_EVENT_HANDLING,
	STRIPE_TYPE_LABELS,
	validateSecret,
} from '#/pages/crm-settings/lib/settings-model'
import type { Settings } from '#/pages/crm-settings/types'
import { SettingsCard } from '#/pages/crm-settings/ui/settings-card'
import { StatusBadge } from '#/pages/customers/ui/badges'

type CopyState = 'idle' | 'copied' | 'manual'

const COPY_MESSAGES: Record<CopyState, string> = {
	idle: '',
	copied: 'URL copiée dans le presse-papiers.',
	manual:
		'Copie impossible ici : l’URL est sélectionnée, copie-la avec Ctrl+C.',
}

function WebhookUrl({ url }: { url: string }) {
	const id = useId()
	const inputRef = useRef<HTMLInputElement>(null)
	const [copy, setCopy] = useState<CopyState>('idle')

	async function copyUrl() {
		try {
			await navigator.clipboard.writeText(url)
			setCopy('copied')
		} catch {
			inputRef.current?.select()
			setCopy('manual')
		}
	}

	return (
		<FormField
			id={id}
			label="URL du webhook"
			hint="À renseigner dans Stripe, rubrique Développeurs puis Webhooks, avec les trois événements ci-dessous."
		>
			<div className="flex gap-2">
				<Input
					id={id}
					ref={inputRef}
					readOnly
					value={url}
					onFocus={(event) => event.target.select()}
					className="font-mono text-[13px]"
				/>
				<Button type="button" variant="outline" onClick={copyUrl}>
					{copy === 'copied' ? <Check aria-hidden /> : <Copy aria-hidden />}
					{copy === 'copied' ? 'Copié' : 'Copier'}
				</Button>
			</div>
			<output className="text-[12px] text-muted-foreground">
				{COPY_MESSAGES[copy]}
			</output>
		</FormField>
	)
}

function SecretForm({
	onClose,
	onSave,
}: {
	onClose: () => void
	onSave: () => Promise<void>
}) {
	const id = useId()
	const [value, setValue] = useState('')
	const [submitted, setSubmitted] = useState(false)
	const error = validateSecret(value)

	function submit(event: FormEvent) {
		event.preventDefault()
		setSubmitted(true)
		if (error) return
		void onSave().then(() => {
			setValue('')
			onClose()
		})
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>Remplacer le secret</DialogTitle>
				<DialogDescription>
					Colle le secret de signature du webhook, copié depuis Stripe. Il sert
					à vérifier les appels reçus, puis n’est plus jamais affiché : seule sa
					présence est conservée.
				</DialogDescription>
			</DialogHeader>
			<FormField
				id={id}
				label="Nouveau secret de signature"
				required
				error={submitted ? (error ?? undefined) : undefined}
			>
				<Input
					id={id}
					type="password"
					autoComplete="new-password"
					spellCheck={false}
					placeholder="whsec_…"
					value={value}
					onChange={(event) => setValue(event.target.value)}
					aria-invalid={submitted && error ? true : undefined}
					aria-describedby={submitted && error ? `${id}-error` : undefined}
				/>
			</FormField>
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit">Remplacer le secret</Button>
			</DialogFooter>
		</form>
	)
}

export function StripeSection({
	organizationSlug,
	settings,
	onReplaceSecret,
}: {
	organizationSlug: string
	settings: Pick<Settings, 'stripeEndpoint' | 'stripeSecretSet'>
	onReplaceSecret: () => Promise<void>
}) {
	const [replacing, setReplacing] = useState(false)
	return (
		<SettingsCard
			id="stripe"
			title="Stripe"
			description="Stripe prévient le CRM à chaque encaissement : l’URL du webhook et le secret de signature relient les deux."
		>
			<div className="grid gap-5 px-5 py-5">
				<WebhookUrl url={settings.stripeEndpoint} />

				<div className="grid gap-1.5">
					<span className="font-medium text-sm">Secret de signature</span>
					<div className="flex flex-wrap items-center gap-3 rounded-md border border-dashed bg-muted/40 px-3 py-2">
						<KeyRound
							className="size-4 shrink-0 text-muted-foreground"
							aria-hidden
						/>
						<span className="font-mono text-[13px] text-muted-foreground">
							{settings.stripeSecretSet
								? SECRET_MASK
								: 'Aucun secret enregistré'}
						</span>
						<StatusBadge tone={settings.stripeSecretSet ? 'ok' : 'warn'}>
							{settings.stripeSecretSet ? 'Configuré' : 'À configurer'}
						</StatusBadge>
						<Button
							type="button"
							variant="outline"
							size="sm"
							className="ml-auto"
							onClick={() => setReplacing(true)}
						>
							{settings.stripeSecretSet
								? 'Remplacer le secret'
								: 'Ajouter le secret'}
						</Button>
					</div>
					<p className="text-[12px] text-muted-foreground">
						Le secret n’est jamais affiché, même à toi.
					</p>
				</div>

				<div>
					<h3 className="mb-2 font-medium text-sm">Événements gérés</h3>
					<div className="overflow-hidden rounded-md border">
						<Table>
							<TableHeader>
								<TableRow className="hover:bg-transparent">
									<TableHead className="pl-3">Événement</TableHead>
									<TableHead className="pr-3">Traitement dans le CRM</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{STRIPE_EVENT_HANDLING.map((entry) => (
									<TableRow key={entry.type}>
										<TableCell className="py-2.5 pl-3 align-top">
											<code className="font-mono text-[13px]">
												{entry.type}
											</code>
											<span className="block text-[12px] text-muted-foreground">
												{STRIPE_TYPE_LABELS[entry.type]}
											</span>
										</TableCell>
										<TableCell className="whitespace-normal pr-3 align-top">
											{entry.summary}
										</TableCell>
									</TableRow>
								))}
							</TableBody>
						</Table>
					</div>
				</div>

				<Link
					to={buildOrgPath(organizationSlug, '/crm/stripe')}
					className="inline-flex w-fit items-center gap-1.5 rounded-sm font-medium text-primary text-sm outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					Voir les événements reçus{' '}
					<ArrowRight className="size-4" aria-hidden />
				</Link>
			</div>

			<Dialog open={replacing} onOpenChange={setReplacing}>
				<DialogContent>
					<SecretForm
						onClose={() => setReplacing(false)}
						onSave={onReplaceSecret}
					/>
				</DialogContent>
			</Dialog>
		</SettingsCard>
	)
}
