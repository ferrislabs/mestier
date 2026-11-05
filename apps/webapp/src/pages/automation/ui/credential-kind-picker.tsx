import { Search } from 'lucide-react'
import { type CSSProperties, useRef, useState } from 'react'
import { Input } from '#/components/ui/input'
import type { AuthScheme } from '#/hooks/use-automation'
import type { ExternalService } from '#/pages/automation/feature/mocks/external-services'
import { credentialAppearance } from '#/pages/automation/lib/credential-appearance'
import {
	schemeDescription,
	searchSchemeGroups,
} from '#/pages/automation/lib/credential-meta'
import { normalizeText } from '#/pages/automation/lib/text'
import { kindTransitionStyle } from '#/pages/automation/ui/credential-kind-names'
import { KindIconTile } from '#/pages/automation/ui/kind-icon-tile'
import '#/pages/automation/ui/automation.css'

function KindCard({
	scheme,
	onPick,
}: {
	scheme: AuthScheme
	onPick: () => void
}) {
	const appearance = credentialAppearance(scheme.kind)
	const iconRef = useRef<HTMLSpanElement>(null)
	const labelRef = useRef<HTMLSpanElement>(null)
	return (
		<button
			type="button"
			onClick={() => {
				for (const [element, part] of [
					[iconRef.current, 'icon'],
					[labelRef.current, 'label'],
				] as const)
					if (element)
						element.style.viewTransitionName =
							kindTransitionStyle(part, scheme.kind).viewTransitionName ?? ''
				onPick()
			}}
			className="flex items-start gap-3 rounded-md border border-border bg-card p-3 text-left outline-none transition-colors hover:border-ring hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
		>
			<span ref={iconRef}>
				<KindIconTile icon={appearance.icon} hue={appearance.hue} size="sm" />
			</span>
			<span className="min-w-0">
				<span ref={labelRef} className="block w-fit font-medium text-sm">
					{scheme.label}
				</span>
				<span className="mt-0.5 block text-muted-foreground text-xs leading-snug">
					{schemeDescription(scheme)}
				</span>
			</span>
		</button>
	)
}

function ServiceCard({ service }: { service: ExternalService }) {
	return (
		<div
			aria-disabled="true"
			className="flex items-center gap-3 rounded-md border border-border border-dashed p-3 opacity-70"
		>
			<span
				className="node-tone flex size-7 shrink-0 items-center justify-center rounded-md border border-(--tone-border) bg-(--tone-bg) font-semibold text-(color:--tone-fg) text-sm"
				style={{ '--node-hue': service.hue } as CSSProperties}
				aria-hidden
			>
				{service.label.charAt(0)}
			</span>
			<span className="min-w-0 flex-1">
				<span className="block font-medium text-sm">{service.label}</span>
				<span className="block truncate text-muted-foreground text-xs">
					{service.description}
				</span>
			</span>
			<span className="shrink-0 rounded-sm bg-muted px-1.5 py-0.5 text-[11px] text-muted-foreground">
				Bientôt
			</span>
		</div>
	)
}

export function CredentialKindPicker({
	schemes,
	services,
	onPick,
}: {
	schemes: readonly AuthScheme[]
	services: readonly ExternalService[]
	onPick: (kind: string) => void
}) {
	const [query, setQuery] = useState('')
	const groups = searchSchemeGroups(schemes, query)
	const needle = normalizeText(query)
	const matchingServices = services.filter(
		(service) =>
			needle === '' ||
			normalizeText(`${service.label} ${service.description}`).includes(needle),
	)

	return (
		<div className="grid gap-4">
			<div className="relative">
				<Search
					className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
					aria-hidden
				/>
				<Input
					type="search"
					value={query}
					onChange={(event) => setQuery(event.target.value)}
					placeholder="Rechercher : OAuth, SMTP, clé d’API, Google…"
					aria-label="Rechercher un type d’identification"
					className="pl-8"
					autoFocus
				/>
			</div>
			{groups.length === 0 && matchingServices.length === 0 && (
				<p className="py-6 text-center text-muted-foreground text-sm">
					Aucun type ne correspond à « {query.trim()} ».
				</p>
			)}
			{groups.map((group) => (
				<section key={group.id} aria-label={group.label} className="grid gap-2">
					<h3 className="font-medium text-muted-foreground text-xs">
						{group.label}
					</h3>
					<div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
						{group.schemes.map((scheme) => (
							<KindCard
								key={scheme.kind}
								scheme={scheme}
								onPick={() => onPick(scheme.kind)}
							/>
						))}
					</div>
				</section>
			))}
			{matchingServices.length > 0 && (
				<section aria-label="Services externes" className="grid gap-2">
					<h3 className="font-medium text-muted-foreground text-xs">
						Services externes
					</h3>
					<p className="text-muted-foreground text-xs">
						Disponibles dès que Mestier prendra en charge la spec OpenAPI de
						chaque service. En attendant, utilisez OAuth2 ou une clé d’API.
					</p>
					<div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
						{matchingServices.map((service) => (
							<ServiceCard key={service.id} service={service} />
						))}
					</div>
				</section>
			)}
		</div>
	)
}
