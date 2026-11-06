import { ArrowLeft, Link2, Lock } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { CredentialKind } from '#/hooks/use-automation'
import { credentialAppearance } from '#/pages/automation/lib/credential-appearance'
import { kindTransitionStyle } from '#/pages/automation/ui/credential-kind-names'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'

export function CredentialKindAside({
	kind,
	usage,
	meta,
	onBack,
}: {
	kind: CredentialKind
	usage: string
	meta?: string
	onBack?: () => void
}) {
	const appearance = credentialAppearance(kind.kind)
	return (
		<aside
			aria-label={`Type : ${kind.label}`}
			className="hidden min-h-0 flex-col gap-5 overflow-y-auto border-border border-r bg-muted/40 p-5 sm:flex"
		>
			{onBack && (
				<Button
					type="button"
					variant="ghost"
					size="sm"
					onClick={onBack}
					className="-ml-2 self-start text-muted-foreground"
				>
					<ArrowLeft />
					Tous les types
				</Button>
			)}
			<div className="grid gap-3">
				<span className="w-fit" style={kindTransitionStyle('icon', kind.kind)}>
					<KindIconTile icon={appearance.icon} hue={appearance.hue} size="lg" />
				</span>
				<div>
					<p
						className="w-fit font-semibold text-base"
						style={kindTransitionStyle('label', kind.kind)}
					>
						{kind.label}
					</p>
					<p className="mt-1 text-muted-foreground text-sm leading-snug">
						{kind.description}
					</p>
				</div>
			</div>
			<p className="flex items-start gap-2 text-muted-foreground text-xs leading-snug">
				<Link2 className="mt-px size-3.5 shrink-0" aria-hidden />
				{usage}
			</p>
			<p className="flex items-start gap-2 text-muted-foreground text-xs leading-snug">
				<Lock className="mt-px size-3.5 shrink-0" aria-hidden />
				Les secrets sont chiffrés à l’enregistrement et ne sont plus jamais
				affichés. Pour en changer un, remplacez-le.
			</p>
			{meta && (
				<p className="mt-auto text-muted-foreground text-xs leading-snug">
					{meta}
				</p>
			)}
		</aside>
	)
}
