package web

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// TestSanitise tests scenarios where unwanted
// characters are added to the URL
func TestSanitise(t *testing.T) {
	plain, ssl, err := Sanitise("")
	require.EqualError(t, err, "web: input is empty")
	require.Equal(t, "", plain)
	require.Equal(t, "", ssl)

	plain, ssl, err = Sanitise("aaa$@%")
	require.Error(t, err)
	require.Equal(t, "", plain)
	require.Equal(t, "", ssl)

	plain, ssl, err = Sanitise("aa@%$^a.com/page")
	require.Error(t, err)
	require.Equal(t, "", plain)
	require.Equal(t, "", ssl)

	plain, ssl, err = Sanitise("aaa")
	require.EqualError(t, err, "web: input is incorrect")
	require.Equal(t, "", plain)
	require.Equal(t, "", ssl)

	plain, ssl, err = Sanitise("aaa.com")
	require.NoError(t, err)
	require.Equal(t, "http://aaa.com", plain)
	require.Equal(t, "https://aaa.com", ssl)

	plain, ssl, err = Sanitise("aaa.com/page")
	require.NoError(t, err)
	require.Equal(t, "http://aaa.com", plain)
	require.Equal(t, "https://aaa.com", ssl)

	plain, ssl, err = Sanitise("http://aaa.com/")
	require.NoError(t, err)
	require.Equal(t, "http://aaa.com", plain)
	require.Equal(t, "", ssl) // http is mentioned

	plain, ssl, err = Sanitise("http://aaa.com/page")
	require.NoError(t, err)
	require.Equal(t, "http://aaa.com", plain)
	require.Equal(t, "", ssl) // http is mentioned

	plain, ssl, err = Sanitise("https://aaa.com/")
	require.NoError(t, err)
	require.Equal(t, "", plain)
	require.Equal(t, "https://aaa.com", ssl) // https is mentioned

	plain, ssl, err = Sanitise("https://aaa.com/page")
	require.NoError(t, err)
	require.Equal(t, "", plain)
	require.Equal(t, "https://aaa.com", ssl) // https is mentioned
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
