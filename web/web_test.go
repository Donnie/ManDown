package web

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// TestSanitise tests scenarios where unwanted
// characters are added to the URL
func TestSanitise(t *testing.T) {
	expected := ""
	found := Sanitise("")
	require.Equal(t, expected, found)

	expected = ""
	found = Sanitise("aaa$@%")
	require.Equal(t, expected, found)

	expected = "https://aaa"
	found = Sanitise("aaa")
	require.Equal(t, expected, found)

	expected = "https://aaa.com"
	found = Sanitise("aaa.com")
	require.Equal(t, expected, found)

	expected = "https://aaa.com"
	found = Sanitise("aaa.com/page")
	require.Equal(t, expected, found)

	expected = "http://aaa.com"
	found = Sanitise("http://aaa.com/page")
	require.Equal(t, expected, found)

	expected = ""
	found = Sanitise("aa@%$^a.com/page")
	require.Equal(t, expected, found)
}

func TestCheckHealth(t *testing.T) {
	expected := Health{
		Site:   "",
		Misc:   "parse \"\": empty url",
		Status: 0,
	}
	found := CheckHealth("")
	require.Equal(t, expected, found)

	expected = Health{
		Site:   "httpz://trust21.com",
		Misc:   "Get \"httpz://trust21.com\": unsupported protocol scheme \"httpz\"",
		Status: 1,
	}
	found = CheckHealth("httpz://trust21.com")
	require.Equal(t, expected, found)

	expected = Health{
		Site:   "https://trustSecretWorld21.com",
		Misc:   "Get \"https://trustSecretWorld21.com\": dial tcp: lookup trustSecretWorld21.com: no such host",
		Status: 1,
	}
	found = CheckHealth("https://trustSecretWorld21.com")
	require.Equal(t, expected.Site, found.Site)
	require.Equal(t, expected.Status, found.Status)
	require.Contains(t, found.Misc, "no such host")

	expected = Health{
		Site:   "https://google.com",
		Status: 200,
	}
	found = CheckHealth("https://google.com")
	require.Equal(t, expected, found)
}
