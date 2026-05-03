import { Loader2 } from 'lucide-react'
import { AnimatePresence, motion } from 'motion/react'
import type React from 'react'
import { Button } from '#/components/ui/button'

export interface FloatingActionBarProps {
	show: boolean
	message?: React.ReactNode
	confirmLabel?: string
	cancelLabel?: string
	onConfirm?: () => void
	onCancel?: () => void
	isLoading?: boolean
	disabled?: boolean
	children?: React.ReactNode
}

export function FloatingActionBar({
	show,
	message = 'Vous avez des modifications non enregistrées',
	confirmLabel = 'Enregistrer',
	cancelLabel = 'Annuler',
	onConfirm,
	onCancel,
	isLoading = false,
	disabled = false,
	children,
}: FloatingActionBarProps) {
	return (
		<AnimatePresence>
			{show ? (
				<motion.div
					key="floating-action-bar"
					initial={{ y: 96, opacity: 0 }}
					animate={{ y: 0, opacity: 1 }}
					exit={{ y: 96, opacity: 0 }}
					transition={{
						type: 'spring',
						stiffness: 400,
						damping: 32,
						mass: 0.8,
					}}
					role="region"
					aria-label="Actions en attente"
					className="pointer-events-none fixed inset-x-0 bottom-4 z-50 flex justify-center px-4 md:bottom-6"
				>
					<div className="pointer-events-auto flex w-full max-w-2xl items-center gap-3 rounded-lg border bg-card/95 p-2 pl-4 shadow-lg backdrop-blur-md">
						<div className="flex flex-1 items-center gap-2 text-sm">
							<span className="relative flex size-2 shrink-0">
								<span className="absolute inline-flex size-full animate-ping rounded-full bg-primary/40" />
								<span className="relative inline-flex size-2 rounded-full bg-primary" />
							</span>
							<span className="truncate text-foreground">{message}</span>
						</div>

						{children ? (
							<div className="flex items-center gap-2">{children}</div>
						) : (
							<div className="flex items-center gap-2">
								{onCancel ? (
									<Button
										variant="ghost"
										size="sm"
										onClick={onCancel}
										disabled={isLoading}
									>
										{cancelLabel}
									</Button>
								) : null}
								{onConfirm ? (
									<Button
										size="sm"
										onClick={onConfirm}
										disabled={disabled || isLoading}
									>
										{isLoading ? <Loader2 className="animate-spin" /> : null}
										{confirmLabel}
									</Button>
								) : null}
							</div>
						)}
					</div>
				</motion.div>
			) : null}
		</AnimatePresence>
	)
}
