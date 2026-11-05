import { KeyRound, Plus, Send, TriangleAlert } from 'lucide-react'
import { type ReactNode, useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { PageHeader, PageShell } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { cn } from '#/lib/utils'
import {
	API_PATH,
	curlExample,
	EXAMPLE_PAYLOAD,
} from '#/pages/support/lib/api-example'
import { formatDateTime, formatRelative } from '#/pages/support/lib/format'
import type { SupportIntegrationUIProps } from '#/pages/support/types'
import { CodeBlock } from '#/pages/support/ui/code-block'

const FIELDS: readonly [string, string, string][] = [
	['subject', 'obligatoire', 'Sujet du ticket.'],
	['body', 'obligatoire', 'Premier message du client.'],
	['requester.email', 'obligatoire', 'Adresse du client, pour lui répondre.'],
	['requester.name', 'facultatif', 'Nom affiché dans le fil de discussion.'],
	[
		'priority',
		'facultatif',
		'low, normal, high ou urgent. Par défaut : normal.',
	],
	[
		'external_id',
		'facultatif',
		'Votre identifiant. Une même valeur ne crée pas de doublon : le ticket existant est renvoyé.',
	],
	[
		'custom_fields',
		'facultatif',
		'Objet libre (catégorie, priorité, référence…). Vos workflows s’en servent pour classer le ticket.',
	],
]

function Card({
	title,
	description,
	actions,
	children,
}: {
	title: string
	description?: string
	actions?: ReactNode
	children: ReactNode
}) {
	return (
		<section
			aria-label={title}
			className="rounded-lg border border-border bg-card"
		>
			<header className="flex flex-wrap items-start justify-between gap-3 border-border border-b px-5 py-4">
				<div>
					<h2 className="font-semibold text-base">{title}</h2>
					{description && (
						<p className="mt-0.5 text-muted-foreground text-sm">
							{description}
						</p>
					)}
				</div>
				{actions}
			</header>
			<div className="grid gap-4 px-5 py-4">{children}</div>
		</section>
	)
}

export function SupportIntegrationUI({
	origin,
	keys,
	revealed,
	testResult,
	isSending,
	onCreateKey,
	onDismissRevealed,
	onRevokeKey,
	onSendTest,
}: SupportIntegrationUIProps) {
	const activeKeys = keys.filter((key) => key.revokedAt === null)
	const [creatingKey, setCreatingKey] = useState(false)
	const [keyName, setKeyName] = useState('')
	const [revokingId, setRevokingId] = useState<string | null>(null)
	const [chosenKeyId, setChosenKeyId] = useState('')
	const [payload, setPayload] = useState(
		JSON.stringify(EXAMPLE_PAYLOAD, null, 2),
	)
	const testKeyId = activeKeys.some((key) => key.id === chosenKeyId)
		? chosenKeyId
		: (activeKeys[0]?.id ?? '')
	const baseId = useId()

	return (
		<PageShell className="gap-0">
			<PageHeader
				className="mb-6"
				title="Intégration API"
				description="Votre plateforme crée des tickets par une simple requête HTTP. Ils arrivent dans le backlog, puis vos workflows les classent."
			/>
			<div className="grid gap-6">
				<Card
					title="Créer un ticket depuis votre plateforme"
					description="Appel à faire côté serveur : la clé ne doit jamais être exposée dans le navigateur de vos utilisateurs."
				>
					<p className="font-mono text-sm">
						<span className="mr-2 rounded-sm bg-primary/10 px-1.5 py-0.5 font-semibold text-primary text-xs">
							POST
						</span>
						{origin}
						{API_PATH}
					</p>
					<CodeBlock
						label="Exemple de requête"
						code={curlExample(origin, activeKeys[0]?.prefix ?? 'mst_live_xxxx')}
					/>
					<div className="overflow-hidden rounded-md border border-border">
						<Table>
							<TableHeader>
								<TableRow className="hover:bg-transparent">
									<TableHead className="px-3 text-muted-foreground">
										Champ
									</TableHead>
									<TableHead className="px-3 text-muted-foreground">
										{' '}
									</TableHead>
									<TableHead className="px-3 text-muted-foreground">
										Rôle
									</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{FIELDS.map(([field, presence, role]) => (
									<TableRow key={field}>
										<TableCell className="px-3 font-mono text-xs">
											{field}
										</TableCell>
										<TableCell className="px-3 text-muted-foreground text-xs">
											{presence}
										</TableCell>
										<TableCell className="whitespace-normal px-3 text-sm">
											{role}
										</TableCell>
									</TableRow>
								))}
							</TableBody>
						</Table>
					</div>
					<ul className="grid gap-1 text-muted-foreground text-sm">
						<li>
							<span className="font-mono text-foreground">201</span> : ticket
							créé, la réponse contient son{' '}
							<span className="font-mono">id</span>, son numéro et son statut.
						</li>
						<li>
							<span className="font-mono text-foreground">200</span> : un ticket
							avec le même <span className="font-mono">external_id</span> existe
							déjà, il est renvoyé tel quel.
						</li>
						<li>
							<span className="font-mono text-foreground">401</span> : clé
							absente, inconnue ou révoquée.{' '}
							<span className="font-mono text-foreground">422</span> : champ
							manquant ou invalide.
						</li>
					</ul>
				</Card>

				<Card
					title="Clés d’API"
					description="Une clé par plateforme. Le secret n’est affiché qu’une fois, à la création."
					actions={
						!creatingKey && (
							<Button variant="outline" onClick={() => setCreatingKey(true)}>
								<Plus />
								Nouvelle clé
							</Button>
						)
					}
				>
					{creatingKey && (
						<form
							className="flex flex-wrap items-end gap-2"
							onSubmit={(event) => {
								event.preventDefault()
								if (keyName.trim() === '') return
								onCreateKey(keyName)
								setKeyName('')
								setCreatingKey(false)
							}}
						>
							<div className="grid min-w-[240px] flex-1 gap-1.5">
								<Label htmlFor={`${baseId}-key-name`}>
									Nom de la plateforme
								</Label>
								<Input
									id={`${baseId}-key-name`}
									value={keyName}
									onChange={(event) => setKeyName(event.target.value)}
									placeholder="Ex. Application partenaires"
									autoFocus
								/>
							</div>
							<Button
								type="button"
								variant="ghost"
								onClick={() => setCreatingKey(false)}
							>
								Annuler
							</Button>
							<Button type="submit" disabled={keyName.trim() === ''}>
								Créer la clé
							</Button>
						</form>
					)}
					{revealed && (
						<output className="grid gap-2 rounded-md border border-state-today/50 bg-state-today/10 p-3">
							<p className="flex items-center gap-2 font-medium text-sm">
								<KeyRound className="size-4" aria-hidden />
								Clé « {revealed.name} » créée
							</p>
							<CodeBlock label="Secret de la clé" code={revealed.secret} />
							<p className="text-sm">
								Copiez-la maintenant et rangez-la dans les secrets de votre
								plateforme : elle ne sera plus jamais affichée.
							</p>
							<Button
								variant="outline"
								size="sm"
								className="justify-self-start"
								onClick={onDismissRevealed}
							>
								J’ai copié la clé
							</Button>
						</output>
					)}
					<div className="overflow-hidden rounded-md border border-border">
						<Table>
							<TableHeader>
								<TableRow className="hover:bg-transparent">
									<TableHead className="px-3 text-muted-foreground">
										Nom
									</TableHead>
									<TableHead className="px-3 text-muted-foreground">
										Préfixe
									</TableHead>
									<TableHead className="px-3 text-muted-foreground">
										Créée le
									</TableHead>
									<TableHead className="px-3 text-muted-foreground">
										Dernier appel
									</TableHead>
									<TableHead className="px-3 text-right text-muted-foreground">
										{' '}
									</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{keys.map((key) => (
									<TableRow
										key={key.id}
										className={cn(key.revokedAt && 'text-muted-foreground')}
									>
										<TableCell className="px-3 font-medium">
											{key.name}
										</TableCell>
										<TableCell className="px-3 font-mono text-xs">
											{key.prefix}…
										</TableCell>
										<TableCell className="px-3 text-sm">
											{formatDateTime(key.createdAt)}
										</TableCell>
										<TableCell className="px-3 text-sm">
											{key.lastUsedAt
												? formatRelative(key.lastUsedAt)
												: 'Jamais'}
										</TableCell>
										<TableCell className="px-3 text-right">
											{key.revokedAt ? (
												<span className="text-sm">Révoquée</span>
											) : revokingId === key.id ? (
												<span className="flex items-center justify-end gap-2">
													<span className="text-sm">
														Les appels échoueront.
													</span>
													<Button
														size="sm"
														variant="ghost"
														onClick={() => setRevokingId(null)}
													>
														Annuler
													</Button>
													<Button
														size="sm"
														variant="destructive"
														onClick={() => {
															onRevokeKey(key.id)
															setRevokingId(null)
														}}
													>
														Révoquer
													</Button>
												</span>
											) : (
												<Button
													size="sm"
													variant="ghost"
													onClick={() => setRevokingId(key.id)}
												>
													Révoquer
												</Button>
											)}
										</TableCell>
									</TableRow>
								))}
							</TableBody>
						</Table>
					</div>
				</Card>

				<Card
					title="Essayer"
					description="Envoie la requête comme le ferait votre plateforme : le ticket est réellement créé et vos règles de classement s’appliquent."
				>
					{activeKeys.length === 0 ? (
						<p className="text-muted-foreground text-sm">
							Créez d’abord une clé d’API.
						</p>
					) : (
						<>
							<div className="grid max-w-sm gap-1.5">
								<Label htmlFor={`${baseId}-test-key`}>Clé utilisée</Label>
								<Select value={testKeyId} onValueChange={setChosenKeyId}>
									<SelectTrigger id={`${baseId}-test-key`} className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{activeKeys.map((key) => (
											<SelectItem key={key.id} value={key.id}>
												{key.name}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</div>
							<div className="grid gap-1.5">
								<Label htmlFor={`${baseId}-payload`}>Corps de la requête</Label>
								<textarea
									id={`${baseId}-payload`}
									value={payload}
									onChange={(event) => setPayload(event.target.value)}
									rows={14}
									spellCheck={false}
									className="w-full resize-y rounded-md border border-input bg-transparent px-3 py-2 font-mono text-xs leading-5 shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
								/>
							</div>
							<div className="flex flex-wrap items-center gap-3">
								<Button
									disabled={isSending}
									onClick={() => onSendTest(testKeyId, payload)}
								>
									<Send />
									Envoyer la requête
								</Button>
								{testResult?.kind === 'invalid' && (
									<div
										role="alert"
										className="flex items-start gap-2 text-sm text-state-alert"
									>
										<TriangleAlert
											className="mt-0.5 size-4 shrink-0"
											aria-hidden
										/>
										<span>
											<span className="font-mono">422</span> :{' '}
											{testResult.errors.join(' ')}
										</span>
									</div>
								)}
								{(testResult?.kind === 'created' ||
									testResult?.kind === 'duplicate') && (
									<output className="text-sm">
										<span className="font-mono">
											{testResult.kind === 'created' ? '201' : '200'}
										</span>{' '}
										:{' '}
										{testResult.kind === 'created'
											? `ticket #${testResult.ticketNumber} créé, statut « Backlog ».`
											: `ce external_id existe déjà, ticket #${testResult.ticketNumber} renvoyé.`}
									</output>
								)}
							</div>
						</>
					)}
				</Card>
			</div>
		</PageShell>
	)
}
