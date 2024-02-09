using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000020 RID: 32
	[HandlerCategory("vvIndicators"), HandlerName("CyberCycle")]
	public class CyberCycle : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000121 RID: 289 RVA: 0x00005ABB File Offset: 0x00003CBB
		public IList<double> Execute(ISecurity src)
		{
			return CyberCycle.GenCyberCycle(src, this.Context, this.Alpha, this.Trigger);
		}

		// Token: 0x0600011F RID: 287 RVA: 0x00005934 File Offset: 0x00003B34
		public static IList<double> GenCyberCycle(ISecurity src, IContext context, double alpha, bool triggerline)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			for (int i = 4; i < count; i++)
			{
				array3[i] = (CyberCycle.P(src, i) + 2.0 * CyberCycle.P(src, i - 1) + 2.0 * CyberCycle.P(src, i - 2) + CyberCycle.P(src, i - 3)) / 6.0;
				array[i] = (1.0 - 0.5 * alpha) * (1.0 - 0.5 * alpha) * (array3[i] - 2.0 * array3[i - 1] + array3[i - 2]) + 2.0 * (1.0 - alpha) * array[i - 1] - (1.0 - alpha) * (1.0 - alpha) * array[i - 2];
				if (i < 8)
				{
					array[i] = (CyberCycle.P(src, i) - 2.0 * CyberCycle.P(src, i - 1) + CyberCycle.P(src, i - 2)) / 4.0;
				}
				array2[i] = array[i - 1];
			}
			if (!triggerline)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x06000120 RID: 288 RVA: 0x00005A96 File Offset: 0x00003C96
		private static double P(ISecurity src, int index)
		{
			return (src.get_HighPrices()[index] + src.get_LowPrices()[index]) / 2.0;
		}

		// Token: 0x1700005E RID: 94
		[HandlerParameter(true, "0.07", Min = "0", Max = "1", Step = "0.01")]
		public double Alpha
		{
			// Token: 0x0600011B RID: 283 RVA: 0x00005912 File Offset: 0x00003B12
			get;
			// Token: 0x0600011C RID: 284 RVA: 0x0000591A File Offset: 0x00003B1A
			set;
		}

		// Token: 0x17000060 RID: 96
		public IContext Context
		{
			// Token: 0x06000122 RID: 290 RVA: 0x00005AD5 File Offset: 0x00003CD5
			get;
			// Token: 0x06000123 RID: 291 RVA: 0x00005ADD File Offset: 0x00003CDD
			set;
		}

		// Token: 0x1700005F RID: 95
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x0600011D RID: 285 RVA: 0x00005923 File Offset: 0x00003B23
			get;
			// Token: 0x0600011E RID: 286 RVA: 0x0000592B File Offset: 0x00003B2B
			set;
		}
	}
}
