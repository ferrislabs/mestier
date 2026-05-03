import type * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '#/lib/utils'

function PageShell({
	className,
	...props
}: React.ComponentPropsWithoutRef<'div'>) {
	return (
		<div
			className={cn(
				'mx-auto flex w-full max-w-7xl flex-col gap-6 p-4 md:p-8',
				className,
			)}
			{...props}
		/>
	)
}

interface PageHeaderProps extends React.ComponentPropsWithoutRef<'header'> {
	title: React.ReactNode
	description?: React.ReactNode
	actions?: React.ReactNode
	eyebrow?: React.ReactNode
	leading?: React.ReactNode
}

function PageHeader({
	title,
	description,
	actions,
	eyebrow,
	leading,
	className,
	children,
	...props
}: PageHeaderProps) {
	return (
		<header
			className={cn(
				'flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between',
				className,
			)}
			{...props}
		>
			{leading ? <div className="shrink-0">{leading}</div> : null}
			<div className="min-w-0">
				{eyebrow ? (
					<p className="mb-1 text-xs font-semibold uppercase text-brand-muted">
						{eyebrow}
					</p>
				) : null}
				<h1 className="truncate text-2xl font-bold text-foreground md:text-[28px]">
					{title}
				</h1>
				{description ? (
					<p className="mt-1 max-w-2xl text-sm leading-6 text-muted-foreground">
						{description}
					</p>
				) : null}
				{children}
			</div>
			{actions ? <div className="shrink-0">{actions}</div> : null}
		</header>
	)
}

function SectionCard({
	className,
	children,
	...props
}: React.ComponentPropsWithoutRef<'section'>) {
	return (
		<section
			className={cn(
				'rounded-lg border bg-card text-card-foreground shadow-xs',
				className,
			)}
			{...props}
		>
			{children}
		</section>
	)
}

interface SectionHeaderProps extends React.ComponentPropsWithoutRef<'div'> {
	title: React.ReactNode
	description?: React.ReactNode
	actions?: React.ReactNode
}

function SectionHeader({
	title,
	description,
	actions,
	className,
	children,
	...props
}: SectionHeaderProps) {
	return (
		<div
			className={cn(
				'flex items-start justify-between gap-4 border-b px-5 py-4',
				className,
			)}
			{...props}
		>
			<div className="min-w-0">
				<h2 className="font-semibold text-foreground">{title}</h2>
				{description ? (
					<p className="mt-0.5 text-xs text-muted-foreground">{description}</p>
				) : null}
				{children}
			</div>
			{actions ? <div className="shrink-0">{actions}</div> : null}
		</div>
	)
}

interface MetricCardProps extends React.ComponentPropsWithoutRef<'div'> {
	label: React.ReactNode
	value: React.ReactNode
	hint?: React.ReactNode
	icon?: React.ReactNode
	trend?: React.ReactNode
}

function MetricCard({
	label,
	value,
	hint,
	icon,
	trend,
	className,
	...props
}: MetricCardProps) {
	return (
		<div
			className={cn(
				'flex min-h-32 flex-col justify-between rounded-lg border bg-card p-5 text-card-foreground shadow-xs',
				className,
			)}
			{...props}
		>
			<div className="flex items-center justify-between gap-3">
				<p className="text-sm font-medium text-muted-foreground">{label}</p>
				{icon ? (
					<span className="flex size-8 items-center justify-center rounded-full bg-brand-soft text-primary">
						{icon}
					</span>
				) : null}
			</div>
			<div>
				<p className="text-3xl font-bold text-foreground md:text-[32px]">
					{value}
				</p>
				{hint || trend ? (
					<p className="mt-1 flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
						{trend ? (
							<span className="font-medium text-success">{trend}</span>
						) : null}
						{hint ? <span>{hint}</span> : null}
					</p>
				) : null}
			</div>
		</div>
	)
}

const statusBadgeVariants = cva(
	'inline-flex shrink-0 items-center rounded-md px-2 py-0.5 text-xs font-medium',
	{
		variants: {
			tone: {
				success: 'bg-success-soft text-success',
				warning: 'bg-warning-soft text-warning',
				error: 'bg-destructive-soft text-destructive',
				neutral: 'bg-muted text-muted-foreground',
				brand: 'bg-brand-soft text-primary',
			},
		},
		defaultVariants: {
			tone: 'neutral',
		},
	},
)

function StatusBadge({
	className,
	tone,
	...props
}: React.ComponentPropsWithoutRef<'span'> &
	VariantProps<typeof statusBadgeVariants>) {
	return (
		<span className={cn(statusBadgeVariants({ tone }), className)} {...props} />
	)
}

const entityAvatarVariants = cva(
	'flex shrink-0 items-center justify-center rounded-lg text-sm font-semibold',
	{
		variants: {
			tone: {
				brand: 'bg-primary text-primary-foreground',
				success: 'bg-success-soft text-success',
				warning: 'bg-warning-soft text-warning',
				neutral: 'bg-muted text-muted-foreground',
			},
			size: {
				sm: 'size-8',
				md: 'size-10',
				lg: 'size-14 text-xl',
			},
		},
		defaultVariants: {
			tone: 'brand',
			size: 'md',
		},
	},
)

function EntityAvatar({
	className,
	tone,
	size,
	...props
}: React.ComponentPropsWithoutRef<'div'> &
	VariantProps<typeof entityAvatarVariants>) {
	return (
		<div
			className={cn(entityAvatarVariants({ tone, size }), className)}
			{...props}
		/>
	)
}

export {
	EntityAvatar,
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
}
