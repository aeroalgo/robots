using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200008B RID: 139
	[HandlerCategory("vvStoch"), HandlerName("DoubleSmoothedStoch")]
	public class DSS : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060004D3 RID: 1235 RVA: 0x0001893C File Offset: 0x00016B3C
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				this.Period.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), this.Period));
			IList<double> data2 = this.Context.GetData("llv", new string[]
			{
				this.Period.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), this.Period));
			IList<double> closePrices = sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			double[] CL2 = new double[closePrices.Count];
			double[] HL2 = new double[closePrices.Count];
			for (int i = 0; i < closePrices.Count; i++)
			{
				CL2[i] = closePrices[i] - data2[i];
				HL2[i] = data[i] - data2[i];
			}
			IList<double> CL2M = this.Context.GetData("ema", new string[]
			{
				this.Period2.ToString(),
				CL2.GetHashCode().ToString()
			}, () => Series.EMA(CL2, this.Period2));
			IList<double> HL2M = this.Context.GetData("ema", new string[]
			{
				this.Period2.ToString(),
				HL2.GetHashCode().ToString()
			}, () => Series.EMA(HL2, this.Period2));
			IList<double> data3 = this.Context.GetData("ema", new string[]
			{
				this.Period1.ToString(),
				CL2M.GetHashCode().ToString()
			}, () => Series.EMA(CL2M, this.Period1));
			IList<double> data4 = this.Context.GetData("ema", new string[]
			{
				this.Period1.ToString(),
				HL2M.GetHashCode().ToString()
			}, () => Series.EMA(HL2M, this.Period1));
			for (int j = 0; j < closePrices.Count; j++)
			{
				array[j] = data3[j] / data4[j] * 100.0;
			}
			return array;
		}

		// Token: 0x170001A6 RID: 422
		public IContext Context
		{
			// Token: 0x060004D4 RID: 1236 RVA: 0x00018C13 File Offset: 0x00016E13
			get;
			// Token: 0x060004D5 RID: 1237 RVA: 0x00018C1B File Offset: 0x00016E1B
			set;
		}

		// Token: 0x170001A3 RID: 419
		[HandlerParameter(true, "5", Min = "1", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x060004CD RID: 1229 RVA: 0x00018867 File Offset: 0x00016A67
			get;
			// Token: 0x060004CE RID: 1230 RVA: 0x0001886F File Offset: 0x00016A6F
			set;
		}

		// Token: 0x170001A4 RID: 420
		[HandlerParameter(true, "10", Min = "5", Max = "100", Step = "5")]
		public int Period1
		{
			// Token: 0x060004CF RID: 1231 RVA: 0x00018878 File Offset: 0x00016A78
			get;
			// Token: 0x060004D0 RID: 1232 RVA: 0x00018880 File Offset: 0x00016A80
			set;
		}

		// Token: 0x170001A5 RID: 421
		[HandlerParameter(true, "20", Min = "5", Max = "100", Step = "5")]
		public int Period2
		{
			// Token: 0x060004D1 RID: 1233 RVA: 0x00018889 File Offset: 0x00016A89
			get;
			// Token: 0x060004D2 RID: 1234 RVA: 0x00018891 File Offset: 0x00016A91
			set;
		}
	}
}
