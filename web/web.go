package web

import (
	"errors"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/asaskevich/govalidator"
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
// to prevent abuse by launching multiple requests
// on a single website.
func Sanitise(site string) (plain string, ssl string, err error) {
	if site == "" {
		// for empty input url.Parse does not throw an error
		err = errors.New("web: input is empty")
		return
	}

	site = strings.ToLower(site)
	if !govalidator.IsURL(site) {
		// for "google" input url.Parse does not throw an error
		err = errors.New("web: input is incorrect")
		return
	}

	// use url.Parse to get structured data from the input
	web, _ := url.Parse(site)
	if web.Host == "" {
		// if scheme is not specified we assume http
		web, _ = url.Parse("http://" + site)
	}

	// when https is specifically mentioned
	// or when nothing is mentioned
	// we add https
	// i.e. when http is not specified
	if !strings.Contains(site, "http://") {
		ssl = "https://" + web.Host
	}

	// when http is specifically mentioned
	// or when nothing is mentioned
	// we add http
	// i.e. when https is not specified
	if !strings.Contains(site, "https://") {
		plain = "http://" + web.Host
	}

	// i.e. when the schema is not input
	// when http and https are both missing
	// then we apply both http and https
	return
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
