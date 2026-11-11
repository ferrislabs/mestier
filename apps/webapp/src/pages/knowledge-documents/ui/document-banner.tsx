import { Lock, TriangleAlert } from 'lucide-react'
import { Button } from '#/components/ui/button'
import type { ConflictKind } from '#/pages/knowledge-documents/lib/document-session'

export type BannerKind =
	| { kind: 'locked'; holderName: string }
	| { kind: 'conflict'; conflict: ConflictKind }
	| { kind: 'failed' }

export function bannerText(banner: BannerKind): string {
	switch (banner.kind) {
		case 'locked':
			return `En cours de modification par ${banner.holderName}`
		case 'conflict':
			return 'Ce document a été modifié ailleurs'
		case 'failed':
			return 'Le document est en lecture seule : l’édition n’a pas pu démarrer'
	}
}

interface DocumentBannerProps {
	banner: BannerKind
	onReload: () => void
}

export function DocumentBanner({ banner, onReload }: DocumentBannerProps) {
	const Icon = banner.kind === 'locked' ? Lock : TriangleAlert
	return (
		<output className="flex flex-wrap items-center gap-3 rounded-lg border border-warning/40 bg-warning-soft px-4 py-2.5 text-sm">
			<Icon aria-hidden className="size-4 shrink-0" />
			<span className="min-w-0 flex-1">{bannerText(banner)}</span>
			<Button type="button" size="sm" variant="outline" onClick={onReload}>
				Recharger
			</Button>
		</output>
	)
}
