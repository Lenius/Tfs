// {{ description }}
import { test, expect } from '@playwright/test';

test.beforeEach(async({ baseUrl, Demo}) => {

})

test.describe('{{ description }}', () => {
{% for s in steps %}
  test.step('{{ s.step }}', async ({ page }) => {
      // TODO: implement "{{ s.step }}"
      // Expected: {{ s.expected }}
  });
{% endfor %}
});