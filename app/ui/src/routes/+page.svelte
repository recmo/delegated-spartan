<script>
	import { invoke } from '@tauri-apps/api/core';
	import { Cog, RocketLaunch, ExclamationCircle, ArrowPath, PaperAirplane } from 'svelte-heros';
	import { humanSize, humanPercentage } from '../lib';

	let sysinfo = invoke('sysinfo');
	$: sysinfo.then((sysinfo) => console.log(JSON.stringify(sysinfo)));

	let gpuinfo = invoke('gpuinfo');
	$: console.log(gpuinfo);

	function refresh() {
		sysinfo = invoke('sysinfo');
		gpuinfo = invoke('gpuinfo');
	}
</script>

<div class="navbar bg-base-100 shadow-md">
	<div class="navbar-start"></div>
	<div class="navbar-center space-x-2">
		<Cog />
		<p class="text-xl">Spartan Benchmark</p>
	</div>
	<div class="navbar-end">
		<button class="btn btn-square btn-ghost" on:click={refresh}>
			<ArrowPath />
		</button>
	</div>
</div>

<div class=" p-3 space-y-4 z-0">
	<div class="hero">
		<div class="hero-content text-center">
			<div class="max-w-md">
				<h1 class="text-5xl font-bold">Spartan</h1>
				<p class="py-6">
					Welcome to Spartan Benchmark! This is a simple benchmarking tool that runs on your system
					to test its performance. Click the button below to start the benchmark.
				</p>
				<button class="btn btn-primary" on:click={() => invoke('run_benchmark')}>
					<RocketLaunch />
					Start Benchmark
				</button>
			</div>
		</div>
	</div>

	<h4 class="font-semibold">System Information</h4>
	<div class="flex flex-wrap gap-3">
		{#await sysinfo}
			<div
				class="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-e-transparent align-[-0.125em] text-surface motion-reduce:animate-[spin_1.5s_linear_infinite] dark:text-white"
				role="status"
			>
				<span
					class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]"
					>Loading...</span
				>
			</div>
		{:then sysinfo}
			<div class="stats shadow">
				<div class="stat">
					<div class="stat-title">Operating System</div>
					<div class="stat-value">{sysinfo.long_os_version}</div>
					<div class="stat-desc">
						Kernel version {sysinfo.kernel_version}
					</div>
				</div>
			</div>

			<div class="stats shadow">
				<div class="stat">
					<div class="stat-title">Memory</div>
					<div class="stat-value">{humanSize(sysinfo.total_memory)}</div>
					<div class="stat-desc">
						{humanPercentage(sysinfo.available_memory / sysinfo.total_memory || 0)} available
					</div>
				</div>
			</div>

			<div class="stats shadow">
				<div class="stat">
					<div class="stat-title">Swap</div>
					<div class="stat-value">{humanSize(sysinfo.total_swap)}</div>
					<div class="stat-desc">
						{humanPercentage(sysinfo.available_swap / sysinfo.total_swap || 0)} available
					</div>
				</div>
			</div>

			{#if sysinfo.cpus}
				<div class="stats shadow">
					<div class="stat">
						<div class="stat-title">CPU</div>
						<!-- <div class="stat-value">{sysinfo.cpus[0].brand}</div> -->
						<div class="stat-desc">
							{sysinfo.physical_core_count} cores at
							<!-- {sysinfo.cpus[0].frequency * 1e-3} GHz -->
						</div>
					</div>
				</div>
			{/if}
		{:catch error}
			<div role="alert" class="alert alert-error">
				<ExclamationCircle />
				<span>Error! {error.message}</span>
			</div>
		{/await}
	</div>
	<h4 class="font-semibold">GPU Information</h4>
	{#await gpuinfo}
		<div
			class="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-e-transparent align-[-0.125em] text-surface motion-reduce:animate-[spin_1.5s_linear_infinite] dark:text-white"
			role="status"
		>
			<span
				class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]"
				>Loading...</span
			>
		</div>
	{:then gpuinfo}
		{#each gpuinfo as gpu}
			<div class="stats shadow">
				<div class="stat">
					<div class="stat-title">{gpu.adapter.backend} GPU</div>
					<div class="stat-value">{gpu.adapter.name}</div>
					<div class="stat-desc">{humanSize(gpu.limits.maxBufferSize)} memory</div>
				</div>
			</div>
		{/each}
	{/await}

	<h4 class="font-semibold">Native Performance</h4>
	<h4 class="font-semibold">Web Performance</h4>

	<button class="btn btn-primary" on:click={() => invoke('run_benchmark')}>
		<PaperAirplane />
		Submit Results
	</button>
</div>
