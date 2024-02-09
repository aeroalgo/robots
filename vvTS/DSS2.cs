using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000089 RID: 137
	[HandlerCategory("vvStoch"), HandlerName("DoubleStoch")]
	public class DSS2 : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060004C0 RID: 1216 RVA: 0x0001849C File Offset: 0x0001669C
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("DoubleStoch", new string[]
			{
				this.StochPeriod.ToString(),
				this.EmaSmoothPeriod.ToString(),
				sec.get_CacheName()
			}, () => DSS2.GenDoubleStoch(sec, this.Context, this.StochPeriod, this.EmaSmoothPeriod));
		}

		// Token: 0x060004BF RID: 1215 RVA: 0x00018248 File Offset: 0x00016448
		public static IList<double> GenDoubleStoch(ISecurity _sec, IContext _ctx, int _StochPeriod, int _EmaSmoothPeriod)
		{
			int count = _sec.get_Bars().Count;
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_StochPeriod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(_sec.get_HighPrices(), _StochPeriod));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_StochPeriod.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(_sec.get_LowPrices(), _StochPeriod));
			IList<double> closePrices = _sec.get_ClosePrices();
			IList<double> arg_C8_0 = _sec.get_HighPrices();
			IList<double> arg_D5_0 = _sec.get_LowPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = _StochPeriod; i < count; i++)
			{
				double num = 2.0 / (1.0 + (double)_EmaSmoothPeriod);
				double num2 = 0.0;
				double num3 = data[i];
				double num4 = data2[i];
				if (num3 != num4)
				{
					num2 = (closePrices[i] - num4) / (num3 - num4) * 100.0;
				}
				array2[i] = array2[i - 1] + num * (num2 - array2[i - 1]);
				num3 = array2[i];
				num4 = array2[i];
				for (int j = 1; j < _StochPeriod; j++)
				{
					num3 = Math.Max(num3, array2[i - j]);
					num4 = Math.Min(num4, array2[i - j]);
				}
				if (num3 != num4)
				{
					num2 = (array2[i] - num4) / (num3 - num4) * 100.0;
				}
				else
				{
					num2 = 50.0;
				}
				array[i] = array[i - 1] + num * (num2 - array[i - 1]);
			}
			return array;
		}

		// Token: 0x1700019F RID: 415
		public IContext Context
		{
			// Token: 0x060004C1 RID: 1217 RVA: 0x00018511 File Offset: 0x00016711
			get;
			// Token: 0x060004C2 RID: 1218 RVA: 0x00018519 File Offset: 0x00016719
			set;
		}

		// Token: 0x1700019E RID: 414
		[HandlerParameter(true, "15", Min = "5", Max = "25", Step = "5")]
		public int EmaSmoothPeriod
		{
			// Token: 0x060004BD RID: 1213 RVA: 0x000181FE File Offset: 0x000163FE
			get;
			// Token: 0x060004BE RID: 1214 RVA: 0x00018206 File Offset: 0x00016406
			set;
		}

		// Token: 0x1700019D RID: 413
		[HandlerParameter(true, "55", Min = "1", Max = "80", Step = "1")]
		public int StochPeriod
		{
			// Token: 0x060004BB RID: 1211 RVA: 0x000181ED File Offset: 0x000163ED
			get;
			// Token: 0x060004BC RID: 1212 RVA: 0x000181F5 File Offset: 0x000163F5
			set;
		}
	}
}
