<?php

declare(strict_types=1);

use App\Http\Controllers\HomeController;
use App\Http\Controllers\WeHypeController;
use Illuminate\Support\Facades\Route;

Route::get('/', [HomeController::class, 'index'])->name('home');
Route::get('/wehype/{id}', [WeHypeController::class, 'show'])->name('wehype');
Route::get('/characters/{character}/preview', [HomeController::class, 'preview'])
    ->whereNumber('character')
    ->name('character.preview');
Route::get('/{character}', [HomeController::class, 'show'])->whereNumber('character')->name('character');
