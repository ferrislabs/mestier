import { Braces, Plus, X } from 'lucide-react'
import { useId, useRef, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import {
	CONTRACT_KINDS,
	type ContractKey,
	type ContractKind,
	contractFromExample,
	contractIssues,
	EXAMPLE_PLACEHOLDERS,
	newContractKey,
	removeContractKey,
	updateContractKey,
} from '#/pages/automation/lib/webhook-contract'
import { TextArea } from '#/pages/automation/ui/text-area'

const GRID =
	'grid grid-cols-[minmax(0,1.3fr)_7rem_3.25rem_minmax(0,1fr)_2rem] items-center gap-x-2'

const EXAMPLE_IMPORT_PLACEHOLDER =
	'{\n  "facture": "FA-2026-0217",\n  "montant": 2340,\n  "client": { "nom": "Claire Fontaine" }\n}'

function ExampleImport({
	onImport,
	onCancel,
}: {
	onImport: (contract: ContractKey[]) => void
	onCancel: () => void
}) {
	const inputId = useId()
	const [text, setText] = useState('')
	const [error, setError] = useState<string | null>(null)

	function submit() {
		try {
			const contract = contractFromExample(JSON.parse(text))
			if (!contract) {
				setError('Collez un objet JSON, par exemple { "montant": 120 }.')
				return
			}
			onImport(contract)
		} catch {
			setError('JSON invalide : vérifiez les guillemets et les virgules.')
		}
	}

	return (
		<div className="grid gap-2 rounded-md border border-border bg-muted/40 p-3">
			<label htmlFor={inputId} className="font-medium text-sm">
				Exemple de corps reçu
			</label>
			<TextArea
				id={inputId}
				rows={7}
				value={text}
				onChange={(event) => {
					setText(event.target.value)
					setError(null)
				}}
				placeholder={EXAMPLE_IMPORT_PLACEHOLDER}
				className="font-mono text-xs"
				autoFocus
			/>
			{error && <p className="text-state-alert text-xs">{error}</p>}
			<p className="text-muted-foreground text-xs">
				Les clés actuelles sont remplacées par les clés de premier niveau de
				l’exemple. Toutes sont marquées requises : ajustez ensuite.
			</p>
			<div className="flex justify-end gap-2">
				<Button type="button" variant="ghost" size="sm" onClick={onCancel}>
					Annuler
				</Button>
				<Button
					type="button"
					size="sm"
					onClick={submit}
					disabled={text.trim() === ''}
				>
					Déduire les champs
				</Button>
			</div>
		</div>
	)
}

export function WebhookContractEditor({
	controlId,
	contract,
	keyErrors,
	onChange,
}: {
	controlId: string
	contract: readonly ContractKey[]
	keyErrors: Readonly<Record<string, string>>
	onChange: (contract: ContractKey[]) => void
}) {
	const [importing, setImporting] = useState(false)
	const ids = useRef<string[]>([])
	while (ids.current.length < contract.length)
		ids.current.push(crypto.randomUUID())
	ids.current.length = contract.length
	const issues = contractIssues(contract)

	return (
		<div id={controlId} className="grid gap-1.5">
			{contract.length > 0 && (
				<div className={`${GRID} px-0.5 text-muted-foreground text-xs`}>
					<span>Clé</span>
					<span>Type</span>
					<span className="text-center">Requis</span>
					<span>Exemple</span>
				</div>
			)}
			{contract.map((key, index) => {
				const issue = issues[index] ?? keyErrors[key.name] ?? null
				const label = key.name || 'nouvelle clé'
				const update = (patch: Partial<ContractKey>) =>
					onChange(updateContractKey(contract, index, patch))
				return (
					<div key={ids.current[index]} className="grid gap-1">
						<div className={GRID}>
							<Input
								value={key.name}
								onChange={(event) => update({ name: event.target.value })}
								placeholder="nom_de_cle"
								aria-label={`Clé ${label}`}
								aria-invalid={issue ? true : undefined}
								className="font-mono text-sm"
							/>
							<Select
								value={key.kind}
								onValueChange={(kind) => update({ kind: kind as ContractKind })}
							>
								<SelectTrigger
									className="w-full"
									aria-label={`Type de ${label}`}
								>
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{CONTRACT_KINDS.map((kind) => (
										<SelectItem key={kind.value} value={kind.value}>
											{kind.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
							<Switch
								checked={key.required}
								onCheckedChange={(required) => update({ required })}
								aria-label={`${label} requis`}
								className="justify-self-center"
							/>
							<Input
								value={key.example ?? ''}
								onChange={(event) => update({ example: event.target.value })}
								placeholder={EXAMPLE_PLACEHOLDERS[key.kind]}
								aria-label={`Exemple pour ${label}`}
								className="text-sm"
							/>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label={`Retirer ${label}`}
								onClick={() => {
									ids.current.splice(index, 1)
									onChange(removeContractKey(contract, index))
								}}
							>
								<X />
							</Button>
						</div>
						{issue && (
							<p role="alert" className="text-state-alert text-xs">
								{issue}
							</p>
						)}
					</div>
				)
			})}
			{contract.length === 0 && !importing && (
				<p className="rounded-md border border-border border-dashed px-3 py-3 text-muted-foreground text-sm">
					Aucun champ décrit : les nœuds suivants ne connaîtront pas la forme du
					corps reçu.
				</p>
			)}
			{importing ? (
				<ExampleImport
					onImport={(next) => {
						ids.current = []
						onChange(next)
						setImporting(false)
					}}
					onCancel={() => setImporting(false)}
				/>
			) : (
				<div className="flex flex-wrap gap-2 pt-1">
					<Button
						type="button"
						variant="outline"
						size="sm"
						onClick={() => onChange([...contract, newContractKey()])}
					>
						<Plus />
						Ajouter un champ
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						onClick={() => setImporting(true)}
					>
						<Braces />
						Déduire d’un exemple JSON
					</Button>
				</div>
			)}
		</div>
	)
}
