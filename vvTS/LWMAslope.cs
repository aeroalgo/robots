using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000183 RID: 387
	[HandlerCategory("vvAverages"), HandlerName("LWMAslope")]
	public class LWMAslope : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000C40 RID: 3136 RVA: 0x000352D4 File Offset: 0x000334D4
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("lwmaslope", new string[]
			{
				this.LWMAperiod.ToString(),
				sec.get_CacheName()
			}, () => this.GenLWMAslope(sec, this.LWMAperiod, this.Context));
		}

		// Token: 0x06000C3F RID: 3135 RVA: 0x00035198 File Offset: 0x00033398
		public IList<double> GenLWMAslope(ISecurity sec, int maperiod, IContext context)
		{
			double[] array = new double[sec.get_Bars().Count];
			IList<double> medpr = context.GetData("MedianPrice", new string[]
			{
				sec.get_CacheName()
			}, () => Series.MedianPrice(sec.get_Bars()));
			IList<double> data = context.GetData("lwma", new string[]
			{
				maperiod.ToString(),
				medpr.GetHashCode().ToString()
			}, () => LWMA.GenWMA(medpr, maperiod));
			for (int i = 1; i < sec.get_Bars().Count; i++)
			{
				array[i] = Math.Pow(sec.get_OpenPrices()[i], 5.0) / Math.Pow(data[i], 5.0);
			}
			return array;
		}

		// Token: 0x17000403 RID: 1027
		public IContext Context
		{
			// Token: 0x06000C41 RID: 3137 RVA: 0x00035338 File Offset: 0x00033538
			get;
			// Token: 0x06000C42 RID: 3138 RVA: 0x00035340 File Offset: 0x00033540
			set;
		}

		// Token: 0x17000402 RID: 1026
		[HandlerParameter(true, "15", Min = "1", Max = "25", Step = "1")]
		public int LWMAperiod
		{
			// Token: 0x06000C3D RID: 3133 RVA: 0x00035159 File Offset: 0x00033359
			get;
			// Token: 0x06000C3E RID: 3134 RVA: 0x00035161 File Offset: 0x00033361
			set;
		}
	}
}
