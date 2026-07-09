<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">

        <title inertia>{{ config('app.name', 'KillFeed') }}</title>

        <!-- SEO -->
        <meta name="description" content="KillFeed turns your EVE Online character's live killmails into an OBS browser-source overlay: killmail cards with ship renders and ISK values, plus ISK destroyed, lost, and efficiency stats.">
        <meta property="og:type" content="website">
        <meta property="og:site_name" content="{{ config('app.name', 'KillFeed') }}">
        <meta property="og:title" content="KillFeed — EVE Online kill overlay for streamers">
        <meta property="og:description" content="Put your kills on stream. A live killfeed overlay for EVE Online streamers, ready for OBS in three steps.">
        <meta property="og:image" content="{{ url('/og.png') }}">
        <meta property="og:url" content="{{ url()->current() }}">
        <meta name="twitter:card" content="summary_large_image">

        <!-- Icons -->
        <link rel="icon" href="/favicon.svg" type="image/svg+xml">
        <link rel="icon" href="/favicon-32.png" type="image/png" sizes="32x32">
        <link rel="apple-touch-icon" href="/apple-touch-icon.png">

        <!-- Fonts -->
        <link rel="preconnect" href="https://fonts.bunny.net">
        <link href="https://fonts.bunny.net/css?family=figtree:400,500,600|chakra-petch:600,700|jetbrains-mono:400,600&display=swap" rel="stylesheet" />

        <!-- Scripts -->
        @vite(['resources/js/app.ts', "resources/js/Pages/{$page['component']}.vue"])
        @inertiaHead
    </head>
    <body class="font-sans antialiased">
        @inertia
    </body>
</html>
