<?php

declare(strict_types=1);

use App\Http\Controllers\HomeController;
use Illuminate\Support\Facades\Route;

Route::get('/', [HomeController::class, 'index'])->name('home');
Route::get('/demo', [HomeController::class, 'demo'])->name('demo');
Route::get('/characters/{character}/preview', [HomeController::class, 'preview'])
    ->whereNumber('character')
    ->name('character.preview');
Route::get('/{character}', [HomeController::class, 'show'])->whereNumber('character')->name('character');
