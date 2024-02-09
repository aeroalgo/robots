using System;
using System.Collections.Generic;
using System.Linq;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000068 RID: 104
	[HandlerCategory("vvIndicators"), HandlerName("Volatility")]
	public class Volatility : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003A1 RID: 929 RVA: 0x0001449C File Offset: 0x0001269C
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			List<double> hlDiff = new List<double>(highPrices.Count);
			for (int i = 0; i < highPrices.Count; i++)
			{
				hlDiff.Add(highPrices[i] - lowPrices[i]);
			}
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				this.EMAPeriod.ToString(),
				hlDiff.GetHashCode().ToString()
			}, () => EMA.GenEMA(hlDiff, this.EMAPeriod));
			double[] array = new double[highPrices.Count];
			for (int j = highPrices.Count - 1; j >= this.ROCPeriod; j--)
			{
				array[j] = (data[j] - data[j - this.ROCPeriod]) / data[j - this.ROCPeriod] * 100.0;
			}
			return array.ToList<double>();
		}

		// Token: 0x17000138 RID: 312
		public IContext Context
		{
			// Token: 0x060003A2 RID: 930 RVA: 0x000145BF File Offset: 0x000127BF
			get;
			// Token: 0x060003A3 RID: 931 RVA: 0x000145C7 File Offset: 0x000127C7
			set;
		}

		// Token: 0x17000136 RID: 310
		[HandlerParameter]
		public int EMAPeriod
		{
			// Token: 0x0600039D RID: 925 RVA: 0x00014458 File Offset: 0x00012658
			get;
			// Token: 0x0600039E RID: 926 RVA: 0x00014460 File Offset: 0x00012660
			set;
		}

		// Token: 0x17000137 RID: 311
		[HandlerParameter]
		public int ROCPeriod
		{
			// Token: 0x0600039F RID: 927 RVA: 0x00014469 File Offset: 0x00012669
			get;
			// Token: 0x060003A0 RID: 928 RVA: 0x00014471 File Offset: 0x00012671
			set;
		}
	}
}
