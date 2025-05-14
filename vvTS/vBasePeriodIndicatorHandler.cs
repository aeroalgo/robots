using System;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000003 RID: 3
	public abstract class vBasePeriodIndicatorHandler
	{
		// Token: 0x17000001 RID: 1
		[HandlerParameter(true, "10", Min = "10", Max = "50", Step = "2")]
		public int Period
		{
			// Token: 0x06000005 RID: 5 RVA: 0x000020CD File Offset: 0x000002CD
			get
			{
				return Math.Max(1, this.m_period);
			}
			// Token: 0x06000006 RID: 6 RVA: 0x000020DB File Offset: 0x000002DB
			set
			{
				this.m_period = value;
			}
		}

		// Token: 0x04000001 RID: 1
		private int m_period;
	}
}
