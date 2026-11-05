import { useState } from 'react'
import { cn } from '#/lib/utils'
import {
	ACTIVITY_GROUPS,
	type HomeActivity,
	itemsOfGroup,
} from '#/pages/home/types'
import { ActivityRow } from '#/pages/home/ui/activity-row'

function GroupCard({ title, items }: { title: string; items: HomeActivity[] }) {
	return (
		<section
			aria-label={title}
			className="overflow-hidden rounded-xl border bg-card"
		>
			<header className="flex items-center justify-between border-b px-4 py-2.5">
				<h3 className="font-medium text-[13px]">{title}</h3>
				<span className="tabular text-[12px] text-muted-foreground">
					{items.length}
				</span>
			</header>
			{items.length === 0 ? (
				<p className="px-4 py-6 text-center text-[13px] text-muted-foreground">
					Rien ici pour le moment.
				</p>
			) : (
				<div className="divide-y">
					{items.map((item) => (
						<ActivityRow key={item.id} item={item} />
					))}
				</div>
			)}
		</section>
	)
}

export function ActivityFeedUI({
	activity,
	isLoading,
}: {
	activity: HomeActivity[]
	isLoading: boolean
}) {
	const [moduleFilter, setModuleFilter] = useState<string | null>(null)
	const items = moduleFilter
		? activity.filter((item) => item.module.id === moduleFilter)
		: activity

	const filterOptions = [
		{ id: null, label: 'Tous' },
		...activity
			.map((item) => item.module)
			.filter(
				(module, index, all) =>
					all.findIndex((other) => other.id === module.id) === index,
			)
			.map((module) => ({ id: module.id, label: module.label })),
	]

	return (
		<section aria-labelledby="home-taches" className="mt-4">
			<div className="flex flex-wrap items-center justify-between gap-3">
				<h2 id="home-taches" className="font-semibold text-base">
					État des tâches
				</h2>
				<fieldset className="flex flex-wrap gap-1">
					<legend className="sr-only">Filtrer par module</legend>
					{filterOptions.map((option) => (
						<button
							key={option.id ?? 'all'}
							type="button"
							aria-pressed={moduleFilter === option.id}
							onClick={() => setModuleFilter(option.id)}
							className={cn(
								'h-7 rounded-full border px-2.5 text-[12px] outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring',
								moduleFilter === option.id
									? 'border-primary bg-primary text-primary-foreground'
									: 'bg-card text-muted-foreground hover:text-foreground',
							)}
						>
							{option.label}
						</button>
					))}
				</fieldset>
			</div>
			<div
				className="mt-4 grid gap-4 lg:grid-cols-2"
				aria-busy={isLoading || undefined}
			>
				{ACTIVITY_GROUPS.map((group) => (
					<GroupCard
						key={group.id}
						title={group.title}
						items={itemsOfGroup(items, group)}
					/>
				))}
			</div>
		</section>
	)
}
