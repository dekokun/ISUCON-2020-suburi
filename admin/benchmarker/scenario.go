package main

import (
	"log"
	"os"
	"sync"
	"time"
)

func voteScenario(wg *sync.WaitGroup, m *sync.Mutex, finishTime time.Time) bool {
	voteSet := setupVotes(50, false)
	resps := map[bool]int{}
	resp := true

	for _, vote := range voteSet {
		resp = postVote(vote)
		resps[resp]++
		if resp == false {
			log.Print("投票に失敗しました at POST /vote")
			os.Exit(1)
		}
	}

	return updateScore("POST", resps, wg, m, finishTime)
}

func invalidVoteScenario(wg *sync.WaitGroup, m *sync.Mutex, finishTime time.Time) bool {
	voteSet := setupVotes(50, false)
	resps := map[bool]int{}
	resp := true

	for _, vote := range voteSet {
		r := getRand(1, 3)
		if r == 1 {
			vote.Name = "hoge"
		} else if r == 2 {
			vote.Address = "hoge"
		} else {
			vote.Mynumber = "hoge"
		}
		resp = postVote(vote)
		resps[resp]++
		if resp == false {
			log.Print("投票に失敗しました at POST /vote")
			os.Exit(1)
		}
	}

	return updateScore("POST", resps, wg, m, finishTime)
}

func indexScenario(wg *sync.WaitGroup, m *sync.Mutex, finishTime time.Time) bool {
	resps := map[bool]int{}
	resp := true

	for i := 0; i < 4; i++ {
		resp = getIndex()
		resps[resp]++
		resp = getCSS()
		resps[resp]++
	}
	return updateScore("GET", resps, wg, m, finishTime)
}

func candidateScenario(wg *sync.WaitGroup, m *sync.Mutex, finishTime time.Time) bool {
	resps := map[bool]int{}
	resp := true

	for i := 0; i < 4; i++ {
		resp = getCandidate()
		resps[resp]++
		resp = getCSS()
		resps[resp]++
	}
	return updateScore("GET", resps, wg, m, finishTime)
}

func politicalPartyScenario(wg *sync.WaitGroup, m *sync.Mutex, finishTime time.Time) bool {
	resps := map[bool]int{}
	resp := true

	for i := 0; i < 4; i++ {
		resp = getPoliticalParty()
		resps[resp]++
		resp = getCSS()
		resps[resp]++
	}
	return updateScore("GET", resps, wg, m, finishTime)
}

// 以下、スコア計算用
func updateScore(method string, resps map[bool]int, wg *sync.WaitGroup, m *sync.Mutex, finishTime time.Time) (finished bool) {
	m.Lock()
	defer m.Unlock()
	if method == "GET" {
		totalScore = totalScore + resps[true]*2
		totalScore = totalScore - resps[false]*100
	} else {
		totalScore = totalScore + resps[true]*1
	}
	totalResp[true] = totalResp[true] + resps[true]
	totalResp[false] = totalResp[false] + resps[false]
	if time.Now().After(finishTime) {
		finished = true
		wg.Done()
	} else {
		finished = false
	}
	return finished
}
