using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200008A RID: 138
	[HandlerCategory("vvStoch"), HandlerName("DoubleStoch Dbl")]
	public class DSS22 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060004C9 RID: 1225 RVA: 0x000187D0 File Offset: 0x000169D0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("DoubleStoch2", new string[]
			{
				this.StochPeriod.ToString(),
				this.EmaSmoothPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => DSS22.GenDoubleStoch2(src, this.Context, this.StochPeriod, this.EmaSmoothPeriod));
		}

		// Token: 0x060004C8 RID: 1224 RVA: 0x0001857C File Offset: 0x0001677C
		public static IList<double> GenDoubleStoch2(IList<double> _src, IContext _ctx, int _StochPeriod, int _EmaSmoothPeriod)
		{
			int count = _src.Count;
			IList<double> data = _ctx.GetData("hhv", new string[]
			{
				_StochPeriod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.Highest(_src, _StochPeriod));
			IList<double> data2 = _ctx.GetData("llv", new string[]
			{
				_StochPeriod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.Lowest(_src, _StochPeriod));
			IList<double> src = _src;
			IList<double> arg_CB_0 = _src;
			IList<double> arg_D3_0 = _src;
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
					num2 = (src[i] - num4) / (num3 - num4) * 100.0;
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

		// Token: 0x170001A2 RID: 418
		public IContext Context
		{
			// Token: 0x060004CA RID: 1226 RVA: 0x0001884E File Offset: 0x00016A4E
			get;
			// Token: 0x060004CB RID: 1227 RVA: 0x00018856 File Offset: 0x00016A56
			set;
		}

		// Token: 0x170001A1 RID: 417
		[HandlerParameter(true, "15", Min = "5", Max = "25", Step = "5")]
		public int EmaSmoothPeriod
		{
			// Token: 0x060004C6 RID: 1222 RVA: 0x0001853B File Offset: 0x0001673B
			get;
			// Token: 0x060004C7 RID: 1223 RVA: 0x00018543 File Offset: 0x00016743
			set;
		}

		// Token: 0x170001A0 RID: 416
		[HandlerParameter(true, "55", Min = "1", Max = "80", Step = "1")]
		public int StochPeriod
		{
			// Token: 0x060004C4 RID: 1220 RVA: 0x0001852A File Offset: 0x0001672A
			get;
			// Token: 0x060004C5 RID: 1221 RVA: 0x00018532 File Offset: 0x00016732
			set;
		}
	}
}
