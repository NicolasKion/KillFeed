<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\Http;
use Inertia\Inertia;
use Inertia\Response;

class WeHypeController extends Controller
{
    public function show(string $id): Response
    {
        $res = Http::post('https://api.wehype.com/creator-community/graphql',
            [
                "operationName" => "GetCreator",
                "query" => "query GetCreator(\$by: CreatorGetByInput!) {\n  creators(by: \$by) {\n    id\n    externalIdentifier\n    backgroundImageUrl\n    displayName\n    games {\n      game {\n        slug\n        name\n        coverImage {\n          transformedUrl(resize: {widthPx: 160, heightPx: 200, mode: FILL})\n        }\n        name\n        id\n      }\n      rewardCycle {\n        status\n        iHaveJoined\n        rewards {\n          id\n          status\n          requiredStreamedMinutes\n          streamedMinutes\n          sku {\n            image {\n              transformedUrl(resize: {widthPx: 200, heightPx: 200, mode: FILL})\n            }\n          }\n          myResult {\n            status\n            rewardedAt\n          }\n        }\n        id\n      }\n      viewerMinutes\n      streamedMinutes\n    }\n  }\n}\n",
                "variables" => [
                    "by" => [
                        "externalIdentifiers" => [
                            $id
                        ]
                    ]
                ]
            ]
        );

        abort_if($res->failed(), 404, 'Creator not found');

        $data = $res->json();

        abort_if(empty($data['data']['creators']), 404, 'Creator not found');

        $creator = $data['data']['creators'][0] ?? null;

        abort_if($creator === null, 404, 'Creator not found');

        return Inertia::render('WeHype', [
            'creator' => $creator,
            'slug' => $id,
        ]);
    }
}
