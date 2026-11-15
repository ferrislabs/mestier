import { ArrowLeft, ArrowRight, Check, CloudOff, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import type { SaveState } from '#/pages/survey-respond/lib/respond-types'

const SAVE_LABELS: Record<Exclude<SaveState, 'error'>, string> = {
	idle: '',
	saving: 'Enregistrement…',
	saved: 'Réponses enregistrées',
}

export function SaveStatus({
	state,
	message,
	onRetry,
}: {
	state: SaveState
	message: string | null
	onRetry: () => void
}) {
	return (
		<div
			aria-live="polite"
			className="flex min-h-6 items-center gap-2 text-xs text-muted-foreground"
		>
			{state === 'saving' ? (
				<Loader2 className="size-3.5 motion-safe:animate-spin" aria-hidden />
			) : null}
			{state === 'saved' ? (
				<Check className="size-3.5 text-success" aria-hidden />
			) : null}
			{state === 'error' ? (
				<>
					<CloudOff className="size-3.5 text-destructive" aria-hidden />
					<span className="text-destructive">
						{message ?? 'Vos réponses n’ont pas été enregistrées.'}
					</span>
					<Button
						type="button"
						variant="link"
						size="xs"
						className="h-auto min-h-11 px-2"
						onClick={onRetry}
					>
						Réessayer
					</Button>
				</>
			) : (
				<span>{SAVE_LABELS[state]}</span>
			)}
		</div>
	)
}

export function RespondActionBar({
	compact = false,
	preview,
	isFirst,
	isLast,
	busy,
	error,
	saveState,
	saveMessage,
	onBack,
	onNext,
	onRetrySave,
}: {
	compact?: boolean
	preview: boolean
	isFirst: boolean
	isLast: boolean
	busy: boolean
	error: string | null
	saveState: SaveState
	saveMessage: string | null
	onBack: () => void
	onNext: () => void
	onRetrySave: () => void
}) {
	return (
		<div
			className={cn(
				compact
					? 'flex flex-col gap-1'
					: 'sticky bottom-0 z-10 -mx-4 mt-auto border-t bg-background/95 px-4 pt-3 pb-[max(0.75rem,env(safe-area-inset-bottom))] backdrop-blur supports-[backdrop-filter]:bg-background/80 sm:-mx-6 sm:px-6',
			)}
		>
			<div
				role="alert"
				className="text-sm font-medium text-destructive empty:hidden"
			>
				{error ?? ''}
			</div>
			{preview ? (
				<p className="flex min-h-6 items-center text-xs text-muted-foreground">
					Aperçu : rien n’est enregistré
				</p>
			) : (
				<SaveStatus
					state={saveState}
					message={saveMessage}
					onRetry={onRetrySave}
				/>
			)}
			<div className={cn('flex', compact ? 'gap-2' : 'mt-2 gap-3')}>
				{isFirst ? null : (
					<Button
						type="button"
						variant="outline"
						className={cn(
							compact
								? 'h-9 min-w-24 rounded-lg text-sm'
								: 'h-12 min-w-28 rounded-xl text-base',
						)}
						disabled={busy}
						onClick={onBack}
					>
						<ArrowLeft className={compact ? 'size-4' : 'size-5'} aria-hidden />
						Précédent
					</Button>
				)}
				<Button
					type="button"
					className={cn(
						compact
							? 'h-9 flex-1 rounded-lg text-sm'
							: 'h-12 flex-1 rounded-xl text-base',
					)}
					aria-busy={busy || undefined}
					disabled={busy}
					onClick={onNext}
				>
					{busy ? (
						<Loader2 className="size-5 motion-safe:animate-spin" aria-hidden />
					) : null}
					{isLast ? 'Terminer' : 'Suivant'}
					{busy || isLast ? null : (
						<ArrowRight className={compact ? 'size-4' : 'size-5'} aria-hidden />
					)}
				</Button>
			</div>
		</div>
	)
}
