import { Info, Lock, TriangleAlert } from 'lucide-react'
import type { ComponentProps, ReactNode } from 'react'
import { cn } from '#/lib/utils'
import { formatCents } from '#/pages/quotes/types'

export function Money({
	cents,
	signed,
	className,
}: {
	cents: number
	signed?: boolean
	className?: string
}) {
	const sign = signed ? (cents > 0 ? '+' : cents < 0 ? '−' : '') : ''
	const value = signed ? Math.abs(cents) : cents
	return (
		<span className={cn('tabular whitespace-nowrap', className)}>
			{sign}
			{sign ? ' ' : ''}
			{formatCents(value)}
		</span>
	)
}

export type LockTone = 'lock' | 'info' | 'warn'

const TONES: Record<LockTone, string> = {
	lock: 'border-border bg-muted/60',
	info: 'border-primary/25 bg-primary/8',
	warn: 'border-state-today/35 bg-state-today/10',
}

const ICON_TONES: Record<LockTone, string> = {
	lock: 'text-muted-foreground',
	info: 'text-primary',
	warn: 'text-state-today',
}

const ICONS = { lock: Lock, info: Info, warn: TriangleAlert }

export function LockBanner({
	title,
	children,
	tone = 'lock',
}: {
	title: string
	children?: ReactNode
	tone?: LockTone
}) {
	const Icon = ICONS[tone]
	return (
		<div
			role="note"
			className={cn(
				'flex items-start gap-3 rounded-lg border px-4 py-3',
				TONES[tone],
			)}
		>
			<Icon
				className={cn('mt-0.5 size-4 shrink-0', ICON_TONES[tone])}
				aria-hidden
			/>
			<div className="grid min-w-0 gap-0.5 text-sm">
				<p className="font-medium">{title}</p>
				{children && <div className="text-muted-foreground">{children}</div>}
			</div>
		</div>
	)
}

function resizeToContent(element: HTMLTextAreaElement) {
	element.style.height = 'auto'
	element.style.height = `${element.scrollHeight}px`
}

export function AutoTextarea({
	className,
	focusOnMount,
	onFocused,
	onChange,
	...props
}: ComponentProps<'textarea'> & {
	focusOnMount?: boolean
	onFocused?: () => void
}) {
	return (
		<textarea
			ref={(element) => {
				if (!element) return
				resizeToContent(element)
				if (focusOnMount) {
					element.focus()
					onFocused?.()
				}
			}}
			rows={1}
			onChange={(event) => {
				resizeToContent(event.currentTarget)
				onChange?.(event)
			}}
			className={cn(
				'block min-h-9 w-full resize-none overflow-hidden rounded-md border border-input bg-transparent px-3 py-1.5 text-sm leading-snug shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:bg-input/30 dark:aria-invalid:ring-destructive/40',
				className,
			)}
			{...props}
		/>
	)
}

export type TileTone = 'default' | 'alert' | 'ok'

const TILE_TONES: Record<TileTone, string> = {
	default: '',
	alert: 'text-state-alert',
	ok: 'text-state-progress',
}

export function StatTile({
	label,
	value,
	hint,
	tone = 'default',
	onClick,
	pressed,
}: {
	label: string
	value: ReactNode
	hint?: ReactNode
	tone?: TileTone
	onClick?: () => void
	pressed?: boolean
}) {
	const content = (
		<>
			<span className="block text-muted-foreground text-sm">{label}</span>
			<span
				className={cn(
					'mt-1 block font-semibold text-2xl tabular-nums',
					TILE_TONES[tone],
				)}
			>
				{value}
			</span>
			{hint && (
				<span className="mt-0.5 block text-muted-foreground text-xs">
					{hint}
				</span>
			)}
		</>
	)
	if (onClick)
		return (
			<button
				type="button"
				onClick={onClick}
				aria-pressed={pressed}
				className={cn(
					'rounded-lg border bg-card px-4 py-3 text-left outline-none transition-colors hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50',
					pressed && 'border-primary bg-primary/5',
				)}
			>
				{content}
			</button>
		)
	return <div className="rounded-lg border bg-card px-4 py-3">{content}</div>
}

export function FormCard({
	id,
	title,
	description,
	children,
}: {
	id: string
	title: string
	description?: string
	children: ReactNode
}) {
	return (
		<section
			aria-labelledby={id}
			className="grid gap-4 rounded-lg border bg-card p-5"
		>
			<div>
				<h2 id={id} className="font-semibold text-base">
					{title}
				</h2>
				{description && (
					<p className="mt-0.5 text-muted-foreground text-sm">{description}</p>
				)}
			</div>
			{children}
		</section>
	)
}

export function OptionCard({
	name,
	value,
	checked,
	disabled,
	onSelect,
	title,
	description,
	children,
}: {
	name: string
	value: string
	checked: boolean
	disabled?: boolean
	onSelect: () => void
	title: string
	description?: ReactNode
	children?: ReactNode
}) {
	return (
		<div
			className={cn(
				'rounded-lg border bg-card p-4 transition-colors has-[input:focus-visible]:ring-[3px] has-[input:focus-visible]:ring-ring/50',
				checked && 'border-primary bg-primary/5',
				disabled && 'opacity-60',
			)}
		>
			<label
				className={cn(
					'flex items-start gap-3',
					disabled ? 'cursor-not-allowed' : 'cursor-pointer',
				)}
			>
				<input
					type="radio"
					name={name}
					value={value}
					checked={checked}
					disabled={disabled}
					onChange={onSelect}
					className="mt-1 size-4 accent-primary outline-none"
				/>
				<span className="grid min-w-0 flex-1 gap-1">
					<span className="font-medium">{title}</span>
					{description && (
						<span className="text-muted-foreground text-sm">{description}</span>
					)}
				</span>
			</label>
			{children && <div className="mt-3 pl-7">{children}</div>}
		</div>
	)
}
