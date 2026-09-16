import { Check, Copy } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

export interface CredentialSecretRevealProps {
	secret: string
	onDone: () => void
}

export function CredentialSecretReveal({
	secret,
	onDone,
}: CredentialSecretRevealProps) {
	const [copied, setCopied] = useState(false)

	return (
		<div className="flex flex-col gap-4">
			<p className="text-sm text-muted-foreground">
				Copiez-le maintenant : il ne sera plus jamais affiché.
			</p>
			<div className="flex flex-col gap-2">
				<Label htmlFor="revealed-secret">Secret</Label>
				<div className="flex gap-2">
					<Input
						id="revealed-secret"
						readOnly
						value={secret}
						className="font-mono"
					/>
					<Button
						type="button"
						variant="outline"
						onClick={() => {
							void navigator.clipboard.writeText(secret)
							setCopied(true)
						}}
					>
						{copied ? <Check /> : <Copy />}
						{copied ? 'Copié' : 'Copier'}
					</Button>
				</div>
			</div>
			<Button type="button" onClick={onDone}>
				Continuer
			</Button>
		</div>
	)
}
