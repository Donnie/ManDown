package web

import (
	"net/http"
	"net/url"
	"strings"
	"time"
)

// Health struct to contain Health of site
type Health struct {
	Site   string
	Status int
	Misc   string
}

// IsAccepted : whether a site is accepted by this bot
func (health *Health) IsAccepted() (out bool) {
	if health.Status != 0 && health.Status != 1 {
		out = true
	}
	return
}

// CheckBulk checks multiple domain healths at once
func CheckBulk(sites []string) []Health {
	ch := make(chan Health)
	var results []Health

	sites = deDupeStr(sites)
	for _, site := range sites {
		go getStatus(site, ch)
	}

	for range sites {
		results = append(results, <-ch)
	}

	return results
}

// getStatus gets the Status code of a single website
func getStatus(site string, ch chan<- Health) {
	web, err := url.ParseRequestURI(site)
	if err != nil {
		ch <- Health{
			Site:   site,
			Misc:   err.Error(),
			Status: 0,
		}
		return
	}

	c := &http.Client{
		Timeout: time.Minute * 3,
	}

	resp, err := c.Get(web.String())
	if err != nil {
		ch <- Health{
			Site:   site,
			Misc:   err.Error(),
			Status: 1,
		}
		return
	}

	status := resp.StatusCode
	ch <- Health{
		Site:   site,
		Status: status,
	}
	return
}

// CheckHealth gets the Status code of one domain
func CheckHealth(site string) Health {
	ch := make(chan Health)
	go getStatus(site, ch)
	result := <-ch

	return result
}

// Sanitise makes sure only the domain name gets through
func Sanitise(site string) string {
	web, err := url.Parse(strings.ToLower(site))
	if err != nil || site == "" {
		return ""
	}

	if web.Scheme != "http" {
		web.Scheme = "https"
	}

	web, _ = url.Parse(web.String())
	return web.Scheme + "://" + web.Hostname()
}

// deDupeStr takes an array of strings and returns only unique strings
func deDupeStr(strs []string) (out []string) {
	for _, item := range strs {
		skip := false
		for _, el := range out {
			if el == item {
				skip = true
				break
			}
		}
		if !skip && item != "" {
			out = append(out, item)
		}
	}
	return out
}
