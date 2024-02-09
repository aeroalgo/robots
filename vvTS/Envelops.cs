using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200011F RID: 287
	[HandlerCategory("vvBands&Channels"), HandlerName("Envelops")]
	public class Envelops : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000830 RID: 2096 RVA: 0x00022D90 File Offset: 0x00020F90
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.Period.ToString(),
				sec.get_ClosePrices().GetHashCode().ToString()
			}, () => Series.SMA(sec.get_ClosePrices(), this.Period));
			double[] array = new double[sec.get_ClosePrices().Count];
			double[] array2 = new double[sec.get_ClosePrices().Count];
			for (int i = 0; i < sec.get_ClosePrices().Count; i++)
			{
				double num = (double)this.K * 0.001;
				array[i] = data[i] * (1.0 + num);
				array2[i] = data[i] * (1.0 - num);
			}
			if (!this.ShowLowBand)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x17000299 RID: 665
		public IContext Context
		{
			// Token: 0x06000831 RID: 2097 RVA: 0x00022EA3 File Offset: 0x000210A3
			get;
			// Token: 0x06000832 RID: 2098 RVA: 0x00022EAB File Offset: 0x000210AB
			set;
		}

		// Token: 0x17000297 RID: 663
		[HandlerParameter(true, "10", Min = "0", Max = "1000", Step = "10")]
		public int K
		{
			// Token: 0x0600082C RID: 2092 RVA: 0x00022D48 File Offset: 0x00020F48
			get;
			// Token: 0x0600082D RID: 2093 RVA: 0x00022D50 File Offset: 0x00020F50
			set;
		}

		// Token: 0x17000296 RID: 662
		[HandlerParameter(true, "14", Min = "2", Max = "20", Step = "2")]
		public int Period
		{
			// Token: 0x0600082A RID: 2090 RVA: 0x00022D37 File Offset: 0x00020F37
			get;
			// Token: 0x0600082B RID: 2091 RVA: 0x00022D3F File Offset: 0x00020F3F
			set;
		}

		// Token: 0x17000298 RID: 664
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool ShowLowBand
		{
			// Token: 0x0600082E RID: 2094 RVA: 0x00022D59 File Offset: 0x00020F59
			get;
			// Token: 0x0600082F RID: 2095 RVA: 0x00022D61 File Offset: 0x00020F61
			set;
		}
	}
}
